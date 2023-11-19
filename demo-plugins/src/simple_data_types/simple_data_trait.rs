use hydrate_pipeline::{DataContainerRef, HandleFactory, PipelineResult};

/// Data that is generally a simple copy from asset to game type, not accessing other assets and not
/// accessing import data. Assets must also implement serialize/deserialize
pub trait SimpleData {
    fn from_data_container(
        data_container: DataContainerRef,
        handle_context: HandleFactory,
    ) -> PipelineResult<Self>
    where
        Self: Sized;
}
