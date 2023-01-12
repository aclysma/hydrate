use std::collections::VecDeque;
use std::ops::Range;
use std::path::{Path, PathBuf};
pub use super::*;

use nexdb::{DataSet, EditorModel, HashMap, HashSet, ObjectId, ObjectLocation, ObjectName, SchemaLinker, SchemaSet, SingleObject, Value};
use type_uuid::TypeUuid;
use serde::{Serialize, Deserialize};
use shaderc::IncludeType;


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


// I'm ignoring that identifiers usually can't start with numbers
pub(crate) fn is_identifier_char(c: char) -> bool {
    if c >= 'a' && c <= 'z' {
    } else if c >= 'A' && c <= 'Z' {
    } else if is_number_char(c) {
    } else if c == '_' {
    } else {
        return false;
    }

    return true;
}

// I'm ignoring that identifiers usually can't start with numbers
pub(crate) fn is_number_char(c: char) -> bool {
    c >= '0' && c <= '9'
}

pub(crate) fn next_non_identifer(
    code: &[char],
    mut position: usize,
) -> usize {
    for i in position..code.len() {
        if !is_identifier_char(code[position]) {
            break;
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

pub(crate) fn try_consume_identifier(
    code: &[char],
    position: &mut usize,
) -> Option<String> {
    let begin = next_non_whitespace(code, *position);

    if begin < code.len() && is_identifier_char(code[begin]) {
        let end = next_non_identifer(code, begin);
        *position = end;
        Some(characters_to_string(&code[begin..end]))
    } else {
        None
    }
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





pub(crate) fn find_included_paths(
    code: &Vec<char>,
) -> Result<HashSet<PathBuf>, String> {
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

















pub struct GlslAsset {}

impl GlslAsset {
    pub fn schema_name() -> &'static str {
        "GlslAsset"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {

            })
            .unwrap();
    }
}

pub struct GlslImportedData {}

impl GlslImportedData {
    pub fn schema_name() -> &'static str {
        "GlslImportedData"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                x.add_string("code");
            })
            .unwrap();
    }
}


#[derive(Serialize, Deserialize)]
struct GlslBuiltData {
    code: String,
}

pub struct GlslAssetPlugin;

impl AssetPlugin for GlslAssetPlugin {
    fn setup(schema_linker: &mut SchemaLinker, importer_registry: &mut ImporterRegistry, builder_registry: &mut BuilderRegistry) {
        GlslAsset::register_schema(schema_linker);
        GlslImportedData::register_schema(schema_linker);

        importer_registry.register_handler::<GlslImporter>(schema_linker);
        builder_registry.register_handler::<GlslBuilder>(schema_linker);
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "d2b0a4ec-5b57-4251-8bd4-affa1755f7cc"]
pub struct GlslImporter;

impl Importer for GlslImporter {
    fn supported_file_extensions(&self) -> &[&'static str] {
        &["glsl", "vert"]
    }

    fn scan_file(&self, path: &Path, schema_set: &SchemaSet) -> Vec<ScannedImportable> {
        log::info!("GlslImporter reading file {:?}", path);
        let code = std::fs::read_to_string(path).unwrap();
        let code: Vec<_> = code.chars().collect();

        let referenced_source_files: Vec<_> = find_included_paths(&code).unwrap().into_iter().map(|path| {
            ReferencedSourceFile {
                importer_id: self.importer_id(),
                path
            }
        }).collect();




        //
        // let mut compile_options = shaderc::CompileOptions::new().unwrap();
        // compile_options.set_include_callback(include::shaderc_include_callback);
        // compile_options.set_optimization_level(shaderc::OptimizationLevel::Performance);
        //
        // let compiler = shaderc::Compiler::new().unwrap();
        // compiler
        //     .preprocess(
        //         &code,
        //         shader_kind,
        //         glsl_file.to_str().unwrap(),
        //         entry_point_name,
        //         Some(&compile_options),
        //     ).unwrap();


        //TODO: Find the include paths

        let asset_type = schema_set.find_named_type(GlslAsset::schema_name()).unwrap().as_record().unwrap().clone();
        vec![ScannedImportable {
            name: None,
            asset_type,
            referenced_source_files
        }]
    }

    fn import_file(
        &self,
        path: &Path,
        object_ids: &HashMap<Option<String>, ObjectId>,
        schema: &SchemaSet,
        //import_info: &ImportInfo,
        referenced_source_file_paths: &mut Vec<PathBuf>,
    ) -> HashMap<Option<String>, SingleObject> {

        let glsl_imported_data_schema = schema
            .find_named_type(GlslImportedData::schema_name())
            .unwrap()
            .as_record()
            .unwrap();

        let mut import_object = SingleObject::new(glsl_imported_data_schema);
        import_object.set_property_override(schema, "code", Value::String("".to_string()));

        let mut imported_objects = HashMap::default();
        imported_objects.insert(None, import_object);
        imported_objects
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "884303cd-3655-4a72-9131-b07b5121ed29"]
pub struct GlslBuilder {}

impl Builder for GlslBuilder {
    fn asset_type(&self) -> &'static str {
        GlslAsset::schema_name()
    }

    fn dependencies(&self, asset_id: ObjectId, data_set: &DataSet, schema: &SchemaSet) -> Vec<ObjectId> {
        vec![asset_id]
    }

    fn build_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>
    ) -> Vec<u8> {
        //
        // Read asset properties
        //
        // let compressed = data_set
        //     .resolve_property(schema, asset_id, "compress")
        //     .unwrap()
        //     .as_boolean()
        //     .unwrap();

        //
        // Read imported data
        //
        let imported_data = &dependency_data[&asset_id];
        let code = imported_data
            .resolve_property(schema, "code")
            .unwrap()
            .as_string()
            .unwrap()
            .to_string();

        let processed_data = GlslBuiltData {
            code,
        };

        let serialized = bincode::serialize(&processed_data).unwrap();
        serialized
    }
}
