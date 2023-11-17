use hydrate_base::hashing::HashMap;
use hydrate_base::AssetId;
use hydrate_data::{DataSet, SchemaSet, SingleObject};
use hydrate_model::pipeline::job_system;
use hydrate_model::pipeline::job_system::*;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use type_uuid::TypeUuid;

//
// Example Job Impl - Imagine this kicking off scatter job(s), and then a gather job that produces the final output
//
#[derive(Hash, Serialize, Deserialize, TypeUuid)]
#[uuid = "512f3024-95a8-4b2e-8b4a-cb1111bac30b"]
pub struct ExampleBuildJobTopLevelInput {
    pub asset_id: AssetId,
}
impl JobInput for ExampleBuildJobTopLevelInput {}

#[derive(Serialize, Deserialize)]
pub struct ExampleBuildJobTopLevelOutput {
    pub final_task: JobId,
}
impl JobOutput for ExampleBuildJobTopLevelOutput {}

#[derive(TypeUuid)]
#[uuid = "2e2c39f2-e672-4d2f-9d22-9e9ff84adf09"]
pub struct ExampleBuildJobTopLevel;

impl JobProcessor for ExampleBuildJobTopLevel {
    type InputT = ExampleBuildJobTopLevelInput;
    type OutputT = ExampleBuildJobTopLevelOutput;

    fn version(&self) -> u32 {
        1
    }

    fn enumerate_dependencies(
        &self,
        context: EnumerateDependenciesContext<Self::InputT>,
    ) -> JobEnumeratedDependencies {
        // No dependencies
        JobEnumeratedDependencies::default()
    }

    fn run(
        &self,
        context: RunContext<Self::InputT>,
    ) -> Self::OutputT {
        let task_id1 = job_system::enqueue_job::<ExampleBuildJobScatter>(
            context.data_set,
            context.schema_set,
            context.job_api,
            ExampleBuildJobScatterInput {
                asset_id: context.input.asset_id,
                some_other_parameter: "Test1".to_string(),
            },
        );
        let task_id2 = job_system::enqueue_job::<ExampleBuildJobScatter>(
            context.data_set,
            context.schema_set,
            context.job_api,
            ExampleBuildJobScatterInput {
                asset_id: context.input.asset_id,
                some_other_parameter: "Test2".to_string(),
            },
        );
        let task_id3 = job_system::enqueue_job::<ExampleBuildJobScatter>(
            context.data_set,
            context.schema_set,
            context.job_api,
            ExampleBuildJobScatterInput {
                asset_id: context.input.asset_id,
                some_other_parameter: "Test3".to_string(),
            },
        );

        let final_task = job_system::enqueue_job::<ExampleBuildJobGather>(
            context.data_set,
            context.schema_set,
            context.job_api,
            ExampleBuildJobGatherInput {
                asset_id: context.input.asset_id,
                scatter_tasks: vec![task_id1, task_id2, task_id3],
            },
        );

        println!("ExampleBuildJobTopLevel");
        ExampleBuildJobTopLevelOutput { final_task }
    }
}

//
// Example Scatter Job Impl
//
#[derive(Hash, Serialize, Deserialize, TypeUuid)]
#[uuid = "122248a9-9350-4ad7-8ef9-ac3795c08511"]
pub struct ExampleBuildJobScatterInput {
    pub asset_id: AssetId,
    pub some_other_parameter: String,
}
impl JobInput for ExampleBuildJobScatterInput {}

#[derive(Serialize, Deserialize)]
pub struct ExampleBuildJobScatterOutput;
impl JobOutput for ExampleBuildJobScatterOutput {}

#[derive(TypeUuid)]
#[uuid = "29755562-5298-4908-8384-7b13b2cedf26"]
pub struct ExampleBuildJobScatter;

impl JobProcessor for ExampleBuildJobScatter {
    type InputT = ExampleBuildJobScatterInput;
    type OutputT = ExampleBuildJobScatterOutput;

    fn version(&self) -> u32 {
        1
    }

    fn enumerate_dependencies(
        &self,
        context: EnumerateDependenciesContext<Self::InputT>,
    ) -> JobEnumeratedDependencies {
        // No dependencies
        JobEnumeratedDependencies::default()
    }

    fn run(
        &self,
        context: RunContext<Self::InputT>,
    ) -> Self::OutputT {
        //Do stuff
        // We could return the result
        // job_api.publish_intermediate_data(...);
        println!("ExampleBuildJobScatter");
        ExampleBuildJobScatterOutput {}
    }
}

//
// Example Gather Job Impl
//
#[derive(Hash, Serialize, Deserialize, TypeUuid)]
#[uuid = "f9b45d02-93ba-44df-8252-555f8e01d0b7"]
pub struct ExampleBuildJobGatherInput {
    pub asset_id: AssetId,
    pub scatter_tasks: Vec<JobId>,
}
impl JobInput for ExampleBuildJobGatherInput {}

#[derive(Serialize, Deserialize)]
pub struct ExampleBuildJobGatherOutput;
impl JobOutput for ExampleBuildJobGatherOutput {}

#[derive(TypeUuid)]
#[uuid = "e5f3de94-2bb6-43a9-bea0-cc91467cdcc3"]
pub struct ExampleBuildJobGather;

impl JobProcessor for ExampleBuildJobGather {
    type InputT = ExampleBuildJobGatherInput;
    type OutputT = ExampleBuildJobGatherOutput;

    fn version(&self) -> u32 {
        1
    }

    fn enumerate_dependencies(
        &self,
        context: EnumerateDependenciesContext<Self::InputT>,
    ) -> JobEnumeratedDependencies {
        JobEnumeratedDependencies {
            import_data: Default::default(),
            upstream_jobs: context.input.scatter_tasks.clone(),
        }
    }

    fn run(
        &self,
        context: RunContext<Self::InputT>,
    ) -> Self::OutputT {
        // Now use inputs from other jobs to produce an output
        //job_api.publish_built_asset(...);

        println!("ExampleBuildJobGather");
        ExampleBuildJobGatherOutput {}
    }
}
