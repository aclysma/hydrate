pub use super::*;
use std::collections::VecDeque;
use std::ops::Range;
use std::path::{Path, PathBuf};

use super::generated::GlslSourceFileImportedDataRecord;
use crate::generated_wrapper::{GlslBuildTargetAssetRecord, GlslSourceFileAssetRecord};
use demo_types::glsl::*;
use hydrate_data::Record;
use hydrate_model::pipeline::Importer;
use hydrate_model::pipeline::{AssetPlugin, Builder, ImportContext, ScanContext};
use hydrate_pipeline::{
    AssetId, BuilderContext, BuilderRegistryBuilder, HashMap, HashSet, ImporterRegistryBuilder,
    JobInput, JobOutput, JobProcessor, JobProcessorRegistryBuilder, PipelineResult, RunContext,
    SchemaLinker,
};
use serde::{Deserialize, Serialize};
use shaderc::IncludeType;
use type_uuid::TypeUuid;

fn range_of_line_at_position(
    code: &[char],
    position: usize,
) -> Range<usize> {
    let mut begin_of_line = position;
    let mut end_of_line = position;

    for i in position..code.len() {
        end_of_line = i + 1;
        if code[i] == '\n' {
            break;
        }
    }

    if position > 0 {
        for i in (0..=position - 1).rev() {
            if code[i] == '\n' {
                break;
            }

            begin_of_line = i;
        }
    }

    begin_of_line..end_of_line
}

pub(crate) fn skip_whitespace(
    code: &[char],
    position: &mut usize,
) {
    *position = next_non_whitespace(code, *position);
}

pub(crate) fn next_non_whitespace(
    code: &[char],
    mut position: usize,
) -> usize {
    for i in position..code.len() {
        match code[position] {
            ' ' | '\t' | '\r' | '\n' => {}
            _ => break,
        }
        position = i + 1;
    }

    position
}

fn remove_line_continuations(code: &[char]) -> Vec<char> {
    let mut result = Vec::with_capacity(code.len());

    let mut previous_non_whitespace = None;
    let mut consecutive_whitespace_character_count = 0;
    for &c in code.iter() {
        match c {
            '\n' => {
                if previous_non_whitespace == Some('\\') {
                    // Pop off any whitespace that came after the \ and the \ itself
                    for _ in 0..=consecutive_whitespace_character_count {
                        result.pop();
                    }

                    consecutive_whitespace_character_count = 0;
                } else {
                    result.push(c);
                }
                previous_non_whitespace = None;
            }
            c @ ' ' | c @ '\t' | c @ '\r' => {
                consecutive_whitespace_character_count += 1;
                result.push(c);
            }
            c @ _ => {
                // Cache what the previous non-whitespace was
                previous_non_whitespace = Some(c);
                consecutive_whitespace_character_count = 0;
                result.push(c);
            }
        }
    }

    result
}

#[derive(Debug)]
pub struct CommentText {
    pub position: usize,
    pub text: Vec<char>,
}

struct RemoveCommentsResult {
    without_comments: Vec<char>,
    comments: VecDeque<CommentText>,
}

fn remove_comments(code: &[char]) -> RemoveCommentsResult {
    let mut in_single_line_comment = false;
    let mut in_multiline_comment = false;
    let mut skip_this_character = false;
    let mut skip_this_character_in_comment_text = false;
    let mut in_string = false;
    let mut without_comments: Vec<char> = Vec::with_capacity(code.len());
    let mut comments = VecDeque::<CommentText>::default();
    let mut comment_text = Vec::<char>::default();
    let mut was_in_comment = false;

    let mut previous_character = None;
    for &c in code.iter() {
        match c {
            '"' => {
                // Begin/end string literals
                if !in_single_line_comment && !in_multiline_comment {
                    in_string = !in_string;
                }
            }
            '\n' => {
                // End single-line comments
                if in_single_line_comment {
                    in_single_line_comment = false;
                    // Don't include the * in the comment text
                    skip_this_character_in_comment_text = true;
                    //skip_this_character = true;
                    // But do add the newline to the code without comments
                    //without_comments.push('\n');
                }
            }
            '/' => {
                if !in_single_line_comment && !in_string {
                    if in_multiline_comment {
                        // End multi-line comment
                        if previous_character == Some('*') {
                            in_multiline_comment = false;
                            // Don't include the / in the resulting code
                            skip_this_character = true;
                            // Remove the * from the comment text
                            comment_text.pop();
                        }
                    } else {
                        // Start a single line comment
                        if previous_character == Some('/') {
                            in_single_line_comment = true;
                            // Remove the / before this
                            without_comments.pop();
                            //// Add a space where comments are to produce correct tokenization
                            //without_comments.push(' ');
                            // Don't include the / in the comment text
                            skip_this_character_in_comment_text = true;
                        }
                    }
                }
            }
            '*' => {
                // Start multi-line comment
                if !in_single_line_comment
                    && !in_multiline_comment
                    && !in_string
                    && previous_character == Some('/')
                {
                    in_multiline_comment = true;
                    // Remove the / before this
                    without_comments.pop();
                    //// Add a space where comments are to produce correct tokenization
                    //without_comments.push(' ');
                    // Don't include the * in the comment text
                    skip_this_character_in_comment_text = true;
                }
            }
            _ => {}
        }

        let in_comment = in_multiline_comment || in_single_line_comment;

        if in_comment && !skip_this_character_in_comment_text {
            comment_text.push(c);
        }

        if !in_comment && !comment_text.is_empty() {
            // If we have comment text we've been accumulating, store it
            let mut text = Vec::default();
            std::mem::swap(&mut text, &mut comment_text);
            comments.push_back(CommentText {
                position: without_comments.len(),
                text,
            });
        }

        if was_in_comment && !in_comment {
            // Add a space where comments are to produce correct tokenization
            without_comments.push(' ');
        }

        if !in_comment && !skip_this_character {
            without_comments.push(c);
        }

        skip_this_character = false;
        skip_this_character_in_comment_text = false;
        previous_character = Some(c);

        if was_in_comment && !in_comment {
            // Hack to handle /**//**/ appearing like a multiline comment and then a single line
            // comment. If we end a comment, then the previous input has been consumed and we should
            // not refer back to it to start a new one.
            previous_character = None;
        }

        was_in_comment = in_comment;
    }

    RemoveCommentsResult {
        without_comments,
        comments,
    }
}

#[derive(PartialEq, Debug)]
struct ParseIncludeResult {
    end_position: usize,
    include_type: IncludeType,
    path: PathBuf,
}

fn try_parse_include(
    code: &[char],
    mut position: usize,
) -> Option<ParseIncludeResult> {
    if position >= code.len() {
        return None;
    }

    if code[position] != '#' {
        // Quick early out, we only do detection if we are at the start of a # directive
        return None;
    }

    // Find start and end of current line
    let line_range = range_of_line_at_position(code, position);

    let first_char = next_non_whitespace(code, line_range.start);
    if position != first_char {
        // We found non-whitespace in front of the #, bail
        None
    } else {
        // Consume the #
        position += 1;

        // Try to find the "include" after the #
        position = next_non_whitespace(code, position);
        if try_consume_literal(code, &mut position, "include").is_some() {
            skip_whitespace(code, &mut position);

            match code[position] {
                '"' => {
                    let end = next_char(code, position + 1, '"');
                    let as_str = characters_to_string(&code[(position + 1)..end]);
                    Some(ParseIncludeResult {
                        end_position: line_range.end,
                        include_type: IncludeType::Relative,
                        path: as_str.into(),
                    })
                }
                '<' => {
                    let end = next_char(code, position + 1, '>');
                    let as_str = characters_to_string(&code[(position + 1)..end]);
                    Some(ParseIncludeResult {
                        end_position: line_range.end,
                        include_type: IncludeType::Standard,
                        path: as_str.into(),
                    })
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

fn next_char(
    code: &[char],
    mut position: usize,
    search_char: char,
) -> usize {
    for i in position..code.len() {
        if code[position] == search_char {
            break;
        }

        position = i + 1;
    }

    position
}

// Return option so we can do .ok_or("error message")?
pub(crate) fn try_consume_literal(
    code: &[char],
    position: &mut usize,
    literal: &str,
) -> Option<()> {
    if is_string_at_position(code, *position, literal) {
        *position += literal.len();
        Some(())
    } else {
        None
    }
}

pub(crate) fn characters_to_string(characters: &[char]) -> String {
    let mut string = String::with_capacity(characters.len());
    for &c in characters {
        string.push(c);
    }

    string
}

pub(crate) fn is_string_at_position(
    code: &[char],
    position: usize,
    s: &str,
) -> bool {
    if code.len() < s.len() + position {
        return false;
    }

    for (index, c) in s.to_string().chars().into_iter().enumerate() {
        if code[position + index] != c {
            return false;
        }
    }

    return true;
}

fn try_consume_preprocessor_directive(
    code: &[char],
    position: usize,
) -> Option<usize> {
    assert!(position < code.len());

    if code[position] != '#' {
        // Quick early out, we only do detection if we are at the start of a # directive
        return None;
    }

    // Find start and end of current line
    let line_range = range_of_line_at_position(code, position);

    let first_char = next_non_whitespace(code, line_range.start);
    if position != first_char {
        // We found non-whitespace in front of the #, bail
        None
    } else {
        //println!("preprocessor directive at {:?}", line_range);
        //print_range(code, &line_range);
        Some(line_range.end)
    }
}

pub(crate) fn find_included_paths(code: &Vec<char>) -> Result<HashSet<PathBuf>, String> {
    let mut paths = HashSet::default();
    let code = remove_line_continuations(&code);
    let remove_comments_result = remove_comments(&code);

    let code = remove_comments_result.without_comments;

    let mut position = 0;
    skip_whitespace(&code, &mut position);

    while position < code.len() {
        //println!("Skip forward to non-whitespace char at {}, which is {:?}", position, code[position]);

        if let Some(new_position) = try_consume_preprocessor_directive(&code, position) {
            let parse_include_result = try_parse_include(&code, position);
            if let Some(parse_include_result) = parse_include_result {
                paths.insert(parse_include_result.path);

                //println!("handle include {:?}", parse_include_result);

                // let included_file = FileToProcess {
                //     path: parse_include_result.path,
                //     include_type: parse_include_result.include_type,
                //     requested_from: file_to_process.path.clone(),
                //     include_depth: file_to_process.include_depth + 1,
                // };

                //parse_shader_source_recursive(&included_file, declarations, included_files)?;

                //println!("finish include");
            }

            position = new_position;
        } else {
            position += 1;
        }

        skip_whitespace(&code, &mut position);
    }

    Ok(paths)
}

pub(crate) fn include_impl<'a>(
    context: &'a RunContext<'a, GlslBuildTargetJobInput>,
    requested_path: &Path,
    include_type: IncludeType,
    requested_from: &Path,
    include_depth: usize,
    dependency_lookup: &HashMap<(PathBuf, PathBuf), AssetId>,
    //dependency_data: &HashMap<AssetId, Arc<SingleObject>>,
) -> PipelineResult<shaderc::ResolvedInclude> {
    log::trace!(
        "include file {:?} {:?} {:?} {:?}",
        requested_path,
        include_type,
        requested_from,
        include_depth,
    );

    // what asset are we calling from?
    // what are the path redirects on it?
    // find the one that matches

    let resolved_path = match include_type {
        IncludeType::Relative => {
            if requested_path.is_absolute() {
                let path = requested_path.to_path_buf();
                log::trace!("absolute path {:?}", path);
                path
            } else {
                let path = requested_from.parent().unwrap().join(requested_path);
                log::trace!("from: {:?} relative path: {:?}", requested_from, path);
                path
            }
        }
        IncludeType::Standard => {
            //TODO: Implement include paths
            requested_from.parent().unwrap().join(requested_path)
        }
    };

    log::trace!(
        "Need to read file {:?} when trying to include {:?} from {:?}",
        resolved_path,
        requested_path,
        requested_from
    );

    let referenced_asset =
        dependency_lookup.get(&(requested_from.to_path_buf(), requested_path.to_path_buf()));
    if let Some(referenced_asset_id) = referenced_asset {
        let dependency_data =
            context.imported_data::<GlslSourceFileImportedDataRecord>(*referenced_asset_id)?;
        let content = dependency_data.code().get()?;
        Ok(shaderc::ResolvedInclude {
            resolved_name: resolved_path.to_str().unwrap().to_string(),
            content: (*content).clone(),
        })
    } else {
        Err(format!(
            "Could not find a file reference for {:?} -> {:?}",
            requested_from, resolved_path
        ))?
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "d2b0a4ec-5b57-4251-8bd4-affa1755f7cc"]
pub struct GlslSourceFileImporter;

impl Importer for GlslSourceFileImporter {
    fn supported_file_extensions(&self) -> &[&'static str] {
        &["glsl", "vert"]
    }

    fn scan_file(
        &self,
        context: ScanContext,
    ) -> PipelineResult<()> {
        log::debug!("GlslSourceFileImporter reading file {:?}", context.path);
        let code = std::fs::read_to_string(context.path)?;
        let code_chars: Vec<_> = code.chars().collect();

        let importable = context.add_default_importable::<GlslSourceFileAssetRecord>()?;

        for include_path in find_included_paths(&code_chars)? {
            importable.add_file_reference_with_importer::<Self, _>(include_path)?;
        }

        Ok(())
    }

    fn import_file(
        &self,
        context: ImportContext,
    ) -> PipelineResult<()> {
        //
        // Read the file
        //
        let code = std::fs::read_to_string(context.path)?;

        //
        // Create import data
        //
        let import_data = GlslSourceFileImportedDataRecord::new_builder(context.schema_set);
        import_data.code().set(code)?;

        let default_asset = GlslSourceFileAssetRecord::new_builder(context.schema_set);
        // Nothing to set

        //
        // Return the created assets
        //
        context
            .add_default_importable(default_asset.into_inner()?, Some(import_data.into_inner()?));
        Ok(())
    }
}

#[derive(Hash, Serialize, Deserialize, Clone)]
pub struct GlslBuildTargetJobInput {
    asset_id: AssetId,
}
impl JobInput for GlslBuildTargetJobInput {}

#[derive(Serialize, Deserialize, Clone)]
pub struct GlslBuildTargetJobOutput {}
impl JobOutput for GlslBuildTargetJobOutput {}

#[derive(Default, TypeUuid)]
#[uuid = "8348dd56-40b5-426b-9d8e-67512d58eee4"]
pub struct GlslBuildTargetJobProcessor;

impl JobProcessor for GlslBuildTargetJobProcessor {
    type InputT = GlslBuildTargetJobInput;
    type OutputT = GlslBuildTargetJobOutput;

    fn version(&self) -> u32 {
        1
    }

    fn run<'a>(
        &self,
        context: &'a RunContext<'a, Self::InputT>,
    ) -> PipelineResult<GlslBuildTargetJobOutput> {
        //
        // Read asset properties
        //
        let asset_data = context.asset::<GlslBuildTargetAssetRecord>(context.input.asset_id)?;

        let source_file = asset_data.source_file().get()?;
        let entry_point = asset_data.entry_point().get()?;

        // We walk through the source file and any files that it includes directly or indirectly
        // We build a queue of files (visit_queue) to visit and track all files that have already
        // been queued
        let mut dependencies = HashSet::default();
        let mut visit_queue = VecDeque::default();
        visit_queue.push_back(source_file);
        dependencies.insert(source_file);

        // Follow references to find all included source files without re-visiting the same file twice
        while let Some(next_reference) = visit_queue.pop_front() {
            let references = context
                .data_set
                .resolve_all_file_references(next_reference)?;

            for (_, &v) in &references {
                if !dependencies.contains(&v) {
                    visit_queue.push_back(v);
                    dependencies.insert(v);
                }
            }
        }

        //
        // Build a lookup of source file AssetID to PathBuf that it was imported from
        //
        let mut dependency_lookup = HashMap::default();
        for dependency_asset_id in dependencies {
            let all_references = context
                .data_set
                .resolve_all_file_references(dependency_asset_id)?;

            let import_info = context
                .data_set
                .import_info(dependency_asset_id)
                .ok_or("Imported GLSL source file had no import info")?;
            let this_path = import_info.source_file_path();
            for (ref_path, ref_obj) in all_references {
                dependency_lookup.insert(
                    (this_path.to_path_buf(), PathBuf::from(ref_path.path)),
                    ref_obj,
                );
            }
        }

        //
        // Compile the shader
        //
        let mut compiled_spv = Vec::default();

        //TODO: Return error if source file not found
        if !source_file.is_null() {
            let source_file_import_info = context
                .data_set
                .import_info(source_file)
                .ok_or("Imported GLSL source file had no import info")?;
            let source_file_import_data =
                &context.imported_data::<GlslSourceFileImportedDataRecord>(source_file)?;
            let code = source_file_import_data.code().get()?;

            let shaderc_include_callback = |requested_path: &str,
                                            include_type: shaderc::IncludeType,
                                            requested_from: &str,
                                            include_depth: usize|
             -> shaderc::IncludeCallbackResult {
                let requested_path: PathBuf = requested_path.into();
                let requested_from: PathBuf = requested_from.into();
                include_impl(
                    &context,
                    &requested_path,
                    include_type.into(),
                    &requested_from,
                    include_depth,
                    &dependency_lookup,
                )
                .map_err(|x| x.to_string())
            };

            let mut compile_options = shaderc::CompileOptions::new().unwrap();
            compile_options.set_include_callback(shaderc_include_callback);
            compile_options.set_optimization_level(shaderc::OptimizationLevel::Performance);
            //NOTE: Could also use shaderc::OptimizationLevel::Size

            let compiler = shaderc::Compiler::new().unwrap();
            let compiled_code = compiler.compile_into_spirv(
                &*code,
                shaderc::ShaderKind::Vertex,
                source_file_import_info.source_file_path().to_str().unwrap(),
                &entry_point,
                Some(&compile_options),
            );

            if let Ok(compiled_code) = compiled_code {
                //println!("SUCCESS BUILDING SHADER");
                compiled_spv = compiled_code.as_binary_u8().to_vec();
            } else {
                panic!("Error: {:?}", compiled_code.err());
            }
        }

        //
        // Create the processed data
        //
        let processed_data = GlslBuildTargetBuiltData { spv: compiled_spv };
        context.produce_default_artifact(context.input.asset_id, processed_data)?;
        Ok(GlslBuildTargetJobOutput {})
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "884303cd-3655-4a72-9131-b07b5121ed29"]
pub struct GlslBuildTargetBuilder {}

impl Builder for GlslBuildTargetBuilder {
    fn asset_type(&self) -> &'static str {
        GlslBuildTargetAssetRecord::schema_name()
    }

    fn start_jobs(
        &self,
        context: BuilderContext,
    ) -> PipelineResult<()> {
        context.enqueue_job::<GlslBuildTargetJobProcessor>(
            context.data_set,
            context.schema_set,
            context.job_api,
            GlslBuildTargetJobInput {
                asset_id: context.asset_id,
            },
        )?;
        Ok(())
    }
}

pub struct GlslAssetPlugin;

impl AssetPlugin for GlslAssetPlugin {
    fn setup(
        importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
        job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        importer_registry.register_handler::<GlslSourceFileImporter>();
        builder_registry.register_handler::<GlslBuildTargetBuilder>();
        job_processor_registry.register_job_processor::<GlslBuildTargetJobProcessor>();
    }
}
