# Build System

In order to build artifacts into assets, there are a few steps required:
 - An asset exists with an associated builder. The builder will create jobs.
   Jobs have arbitrary inputs/outputs and can be dependent on each other.
 - Jobs can produce one or more artifacts
 - Produced artifacts are written to disk and loaded at runtime by UUID or
   symbol name

## Builders

Builders act as an entry point to a chain of work that produces one or more
artifacts. The common case is that a builder just creates a single job and returns.

```rust
#[derive(TypeUuid, Default)]
#[uuid = "da6760e7-5b24-43b4-830d-6ee4515096b8"]
pub struct GpuImageBuilder {}

impl Builder for GpuImageBuilder {
    fn asset_type(&self) -> &'static str {
        GpuImageAssetRecord::schema_name()
    }

    fn start_jobs(
        &self,
        context: BuilderContext,
    ) -> PipelineResult<()> {
        context.enqueue_job::<GpuImageJobProcessor>(
            context.data_set,
            context.schema_set,
            context.job_api,
            GpuImageJobInput {
                asset_id: context.asset_id,
            },
        )?;

        Ok(())
    }
}
```

## Jobs

Unlike builders, which are fairly limited in what they can do, jobs can support
an extensive variety of work patterns. The common case is a single builder,
starting a single job, producing a single asset. However, jobs can access
any assets, import data for any number of assets, and the results of other jobs.
They can produce data for other jobs, and produce any number of assets.

This allows for interesting kinds of operations such as compacting a large
collection of assets into a fast lookup structure, or using memoized result
of a very expensive operation to quickly produce many variants of an asset.

Jobs have `enumerate_dependencies()` and `run()` methods. `enumerate_dependencies`
only needs to be implemented if you with to ensure a job runs after some other
job.

```rust
#[derive(Hash, Serialize, Deserialize)]
pub struct GpuImageJobInput {
    pub asset_id: AssetId,
}
impl JobInput for GpuImageJobInput {}

#[derive(Serialize, Deserialize)]
pub struct GpuImageJobOutput {}
impl JobOutput for GpuImageJobOutput {}

#[derive(Default, TypeUuid)]
#[uuid = "5311c92e-470e-4fdc-88cd-3abaf1c28f39"]
pub struct GpuImageJobProcessor;

impl JobProcessor for GpuImageJobProcessor {
    type InputT = GpuImageJobInput;
    type OutputT = GpuImageJobOutput;

    fn version(&self) -> u32 {
        1
    }

    fn run<'a>(
        &'a self,
        context: &'a RunContext<'a, Self::InputT>,
    ) -> PipelineResult<GpuImageJobOutput> {
       // ...
       context.produce_default_artifact(context.input.asset_id, processed_data)?;

       Ok(GpuImageJobOutput {})
    }
}
```

Note that only one default artifact can be produced per asset. To produce more than
one artifact, use `produce_artifact()`. This function takes a hashable key, which becomes
part of the identifier for the artifact.

If an artifact needs to reference another artifact, it can do so with a `Handle<T>`. The
`produce_artifact_with_handles()` or `produce_default_artifact_with_handles()` calls
provide a callback that passes you a handle factory. You can use this factory to produce
handles that can serialize within the artifact data.

Builders can also throw warnings and errors by calling `warn()` or `error()` on the context.

## Registration
Builders (like jobs and importers) are registred with an asset plugin

```rust
pub struct GpuImageAssetPlugin;

impl AssetPlugin for GpuImageAssetPlugin {
    fn setup(context: AssetPluginSetupContext) {
        context
            .builder_registry
            .register_handler::<GpuImageBuilder>();
        context
            .job_processor_registry
            .register_job_processor::<GpuImageJobProcessor>();
    }
}
```


