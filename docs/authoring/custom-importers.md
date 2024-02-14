# Custom Importers

In order to consume game data authored in other tools, such as blender or photoshop,
exported data must be exported to disk and then imported to hydrate. Hydrate supports
a wide range of import scenarios:

 - Single Source File -> Single Asset
   - Example: A .png file producing an image asset
 - Single Source File -> Multiple assets of different types
   - Example: A GLTF file including textures, meshes, animations, etc.
 - Many source files -> single assets
   - Technically, even in this case each file must produce an asset. However, the
     editor is capable of following paths from a single source file and importing
     an interconnected group of files in a single import. Build steps can walk across
     this data to produce sinle artifacts.
   - Example: GLSL source code

Imports generally happen for one of three reasons:
 - A file is drag and dropped into the editor
 - A file has changed and the user clicked a button to re-import
 - On initial load, any source files in a path-based data source will be automatically imported.

## Importer API

In order to support automatic discovery/import of source files referenced by other
source files, and provide a UI to only import part of a complex file format, the import
API follows a two-step process:
 - Scan: A function that describes the contents of a source file, such as available
   assets to import and path references to other source files.
 - Import: Using the metadata provided by the scan(), import one or more assets from
   the source file(s.)

```rust
#[derive(TypeUuid, Default)]
#[uuid = "e7c83acb-f73b-4b3c-b14d-fe5cc17c0fa3"]
pub struct GpuImageImporter;

impl Importer for GpuImageImporter {
    fn supported_file_extensions(&self) -> &[&'static str] {
        &["png", "jpg", "tif"]
    }

    fn scan_file(
        &self,
        context: ScanContext,
    ) -> PipelineResult<()> {
        // ...
    }

    fn import_file(
        &self,
        context: ImportContext,
    ) -> PipelineResult<()> {
        // ...
    }
}

// Register with AssetPluginRegistryBuilders
pub struct GpuImageAssetPlugin;

impl AssetPlugin for GpuImageAssetPlugin {
    fn setup(context: AssetPluginSetupContext) {
        context
            .importer_registry
            .register_handler::<GpuImageImporter>();
    }
}

```
