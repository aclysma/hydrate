# Project Configuration

When hydrate tools load (editor, code generation, etc.), they travers up the file system from the current directory looking for a `hydrate_project.json` file. 

Here is an example project configuration:

```json
{
  "schema_def_paths": [
    "demo-editor/data/schema"
  ],
  "import_data_path": "demo-editor/data/import_data",
  "build_data_path": "demo-editor/data/build_data",
  "job_data_path": "demo-editor/data/job_data",
  "id_based_asset_sources": [
    {
      "name": "vault",
      "path": "demo-editor/data/assets_id_based"
    }
  ],
  "path_based_asset_sources": [
    {
      "name": "assets",
      "path": "demo-editor/data/assets_path_based"
    }
  ],
  "source_file_locations": [
    {
      "name": "art",
      "path": "demo-editor/data/art"
    }
  ],
  "schema_codegen_jobs": [
    {
      "name": "demo",
      "schema_path": "demo-editor/data/schema",
      "included_schema_paths": [],
      "outfile": "demo-plugins/src/generated.rs"
    }
  ]
}

```

## Reference

 - `schema_def_paths: [<paths>]`: A list of directories that contain schema files. Schemas in a directory may reference types by name that are contained in other schema directories.
 - `import_data_path: <path>`: The location of all import data. Import data is any data that is imported in the editor and is not editable directly. This data should generally be checked into source control. Deleting this data may cause imported assets to no longer be usable.
 - `build_data_path: <path>:`: The output location for all build data. This data should generally *not* be committed to source control. It should always be safe to delete the contents of this folder and rebuild.
 - `job_data_path: <path>`: Location for cached intermediate build data. This data should *not* be committed to source control. It should always be safe to delete the contents of this folder and rebuild.
 - `id_based_asset_sources: [{name: string, path: <path>}]`: Location of assets that are stored based on UUID. If you use an ID-based data source, objects can be moved and renamed freely without concern of broken asset references. Source files are *not* imported automatically. This is a great choice for data that is purely authored in-engine. However, you do not *have* to use this kind of data source.
 - `path_based_data_sources: [{name: string, path: <path>}]`: Location of assets that are stored based on path. Any source files stored in a path based data source are automatically imported when the editor is launched.
 - `source_file_locations: [{name: string, path: <path>}]`: Location of source files (png, gltf, etc.) that are frequently imported. While you may import data from anywhere on disk, importing from a named location avoids dependence on paths that may include your username. These locations will usually be committed to source control and may be directories artists frequently export to.
 - `schema_codegen_jobs: [{...}]`: The codegen tool can either be configured with command line arguments or by referencing a particular job by name here. This is a convenience option to ensure that everyone on a team is using the codegen tool consistently.