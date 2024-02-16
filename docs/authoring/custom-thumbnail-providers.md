# Custom Thumbnail Providers

Producing thumbnails in the editor is a surprisingly complex problem. First, it's likely
that import data will need to be available in order to create a thumbnail. (As an example,
for an image.) It's even possible that an asset's thumbnail will change depending on
asset settings. Ensuring that thumbnails are re-created with important changes to the
asset state occur is tricky.

Further, the editor needs a way to know that the thumbnail should be invalidated without
having to actually re-create the thumbnail.

To support this, custom thumbnail providers must implement a `gather()` and a `render()`
function. The `gather()` function will make calls on the context to request the information
it needs (such as import data.) The requested information is hashed, and if the hash changes,
the thumbnail is invalidated and recreated.

```rust
impl ThumbnailProvider for GpuImageThumbnailProvider {
    type GatheredDataT = ();
    
    // ...
    
    fn gather(
        &self,
        context: ThumbnailProviderGatherContext,
    ) -> Self::GatheredDataT {
        context.add_import_data_dependency(context.asset_id);
    }

    fn render<'a>(
        &'a self,
        context: &'a ThumbnailProviderRenderContext<'a>,
        _gathered_data: Self::GatheredDataT,
    ) -> PipelineResult<ThumbnailImage> {
        let import_data = context.imported_data::<GpuImageImportedDataRecord>(context.asset_id)?;
        // ...
    }
}
```