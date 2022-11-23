use uuid::Uuid;
use crate::{DataSet, DataSource, Schema, SchemaSet};
use crate::edit_context::EditContext;

/*
// Returned by an active job's update() call
enum AssetProcessorJobResult {
    Running, // return progress
    Finished, // return results
    Failed(Vec<String>) // return results, errors, warnings
}

// An actively running job. Update will be called until it returns Finished for Failed state
trait AssetProcessorJob {
    // Called frequently until it's finished. Can be used to read main thread state
    fn update(&mut self) -> AssetProcessorJobUpdateResult;
}

// Unique ID for a job. The same job spawned over and over should produce the same ID
struct JobId(u128);

// When we first iterate over all our assets to find jobs to trigger, we produce metadata necessary
// to schedule the jobs
struct AssetJobScheduleMeta {
    id: JobId,
    dependencies: Vec<JobId>,
}

// A trait that can be implemented to add new kinds of jobs. Is used to collect jobs to be
// scheduled and to start the job
trait AssetJobFactory {
    fn create_jobs(&self, schema: &SchemaSet, dataset: &DataSet) -> Vec<AssetJobScheduleMeta>;

    fn start_job(&self, schema: &SchemaSet, dataset: &DataSet, meta: AssetJobMeta) -> Box<dyn AssetProcessorJob>;
}

// Example implementation of a job
struct ImageAssetJobFactory {

}

impl AssetJobFactory for ImageAssetJobFactory {
    fn create_jobs(&self, schema: &SchemaSet, dataset: &DataSet) -> Vec<PendingJob> {
        let mut jobs = Vec::default();
        let ty = schema.find_named_type("ImageAsset").unwrap();

        for (id, object) in &dataset.objects {
            if object.schema.fingerprint() == ty.fingerprint() {
                jobs.push(PendingJob {
                    id: JobId(id.0),
                    dependencies: Vec::default()
                })
            }
        }

        jobs
    }

    fn start_job(&self, schema: &SchemaSet, dataset: &DataSet, meta: AssetJobMeta) -> Box<dyn AssetProcessorJob> {
        unimplemented!()
    }
}


struct AssetProcessor {

}

impl AssetProcessor {

}


fn process_assets() {
    // Find all jobs with their dependencies

    // build dependency graph

    // first-pass: just linearize the graph
    // future: priority-queue with key = number of dependencies?
}
*/