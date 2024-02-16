# Loading Artifacts at Runtime

In order to load assets, hydrate uses a few concepts:
 - LoaderIO: Reads data, likely from disk. A custom IO could read from a web service, for example,
 - Loader: Tracks loaded assets and makes requests of the LoaderIO to instantiate artifacts into memory.
 - Storage: Contains loaded artifact data. The Loader writes to storage, and the game reads from it (using handles.)
 - Handles: Any reference to a loaded artifact is via a handle.

There are two kinds of handles:
 - Indirect: Any handle created via API is an indirect handle. Indirect handles may be redirected
   to new data if artifact data is rebuilt and hot-reloaded.
 - Direct: Any handle stored in a loaded artifact is a direct handle.

Generally speaking, users do not need to treat a direct or indirect handle any different.
Just keep in mind that if you clone a direct handle, it will not update when data is
hot-reloaded.

## LoaderIO

Hydrate includes a reference LoaderIO that directly reads loose binary data produced
by building. A shipped game will likely want to replace this with some kind of
packaging solution that combines the binary data into fewer, larger files.

## Loader

Hydrate includes a reference Loader that would be sufficient for many games. That
said, advanced streaming scenarios or a slimmed down loader that does not support
hot-reloading might be worth implementing in a shipped game.

## Storage

The loader writes data into Storage for the various asset types that are registered.
Storage can optionally be registered with a loader (likely to be renamed, to avoid
confusion with the previously mentioned Loader) that allows for custom initialization
logic. The most likely use for this is initializing GPU resources.

When storage has a custom loader, it is passed an `ArtifactLoadOp`. This struct should
be held in the loader until the artifact either finishes initializing or fails to
initialize. Once either of these events occur, call `complete()` or `error()` on
the load op.

See the demo-game crate for examples.

## AssetManager

For convenience, hydrate includes an AssetManager that sets up IO, Loader, and Storage
automatically. All that is required at runtime is to register artifact storages and
request handles.

```rust
let mut artifact_manager = hydrate::loader::ArtifactManager::new(build_data_source_path).unwrap();
artifact_manager.add_storage_with_loader::<GpuImageAssetData, GpuImageAsset, GpuImageLoader>(Box::new(
    GpuImageLoader
));
artifact_manager.add_storage::<GpuBufferBuiltData>();
artifact_manager.add_storage::<Transform>();

let load_handle_transform_ref: Handle<TransformRef> =
    artifact_manager.load_artifact_symbol_name("assets://test_transform_ref");
let load_handle_image: Handle<GpuImageAsset> =
    artifact_manager.load_artifact_symbol_name("assets://test_texture.jpg");

loop {
    artifact_manager.update();
    
    if let Some(data) = load_handle_transform_ref.artifact(artifact_manager.storage()) {
        println!("load_handle_transform_ref loaded {:?}", data);
    } else {
        println!("not loaded");
    }
}

```