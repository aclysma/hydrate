# Recommended Starting Project Layout

The following project layout is suggested as a good starting point:

```
your_game/
  content/
    art/ - Source data created by artists/designers (photoshop, blender, etc.)
    assets/ - Import-ready data and in-engine authored data (gltf, png, etc.)
    import_data/ - Will be used to store imported data
  pipeline/
    build_data/ - Will be the output directory of builds
    job_data/ - Will be the intermediate directory of builds
  schema/ - Custom data types are defined here
  hydrate_project.json
  [Other files necessary for development of the game, like source code]
```

## hydrate_project.json

Matching hydrate_project.json file

```json
{
  "schema_def_paths": [
    "schema"
  ],
  "import_data_path": "content/import_data",
  "build_data_path": "pipeline/build_data",
  "job_data_path": "pipeline/job_data",
  "id_based_asset_sources": [],
  "path_based_asset_sources": [
    {
      "name": "assets",
      "path": "content/assets"
    }
  ],
  "source_file_locations": [
    {
      "name": "art",
      "path": "content/art"
    }
  ],
  "schema_codegen_jobs": [
    {
      "name": "default",
      "schema_path": "schema",
      "included_schema_paths": [],
      "outfile": "src/generated.rs"
    }
  ]
}
```

