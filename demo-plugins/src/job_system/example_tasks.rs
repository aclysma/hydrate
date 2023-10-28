use std::hash::Hash;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use siphasher::sip128::Hasher128;
use type_uuid::{Bytes, TypeUuid};
use uuid::Uuid;
use hydrate_base::hashing::HashMap;
use hydrate_base::ObjectId;
use hydrate_data::{DataSet, SchemaSet, SingleObject};
use hydrate_model::BuiltAsset;

use super::traits::*;






//
// Example Job Impl - Imagine this kicking off scatter job(s), and then a gather job that produces the final output
//
#[derive(Hash, Serialize, Deserialize, TypeUuid)]
#[uuid = "512f3024-95a8-4b2e-8b4a-cb1111bac30b"]
pub struct ExampleBuildJobTopLevelInput {
    pub asset_id: ObjectId,
}
impl BuildJobInput for ExampleBuildJobTopLevelInput {}

#[derive(Serialize, Deserialize)]
pub struct ExampleBuildJobTopLevelOutput {
    pub final_task: Uuid
}
impl BuildJobOutput for ExampleBuildJobTopLevelOutput {}

#[derive(TypeUuid)]
#[uuid = "2e2c39f2-e672-4d2f-9d22-9e9ff84adf09"]
pub struct ExampleBuildJobTopLevel;

impl BuildJobWithInput for ExampleBuildJobTopLevel {
    type InputT = ExampleBuildJobTopLevelInput;
    type OutputT = ExampleBuildJobTopLevelOutput;

    fn enumerate_dependencies(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> BuildJobRunDependencies {
        // No dependencies
        BuildJobRunDependencies::default()
    }

    fn run(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        build_job_api: &dyn BuildJobApi
    ) -> Self::OutputT {
        let task_id1 = super::traits::enqueue_build_task::<ExampleBuildJobScatter>(build_job_api, data_set, schema_set, ExampleBuildJobScatterInput {
            asset_id: input.asset_id,
            some_other_parameter: "Test1".to_string()
        });
        let task_id2 = super::enqueue_build_task::<ExampleBuildJobScatter>(build_job_api, data_set, schema_set, ExampleBuildJobScatterInput {
            asset_id: input.asset_id,
            some_other_parameter: "Test2".to_string()
        });
        let task_id3 = super::enqueue_build_task::<ExampleBuildJobScatter>(build_job_api, data_set, schema_set, ExampleBuildJobScatterInput {
            asset_id: input.asset_id,
            some_other_parameter: "Test3".to_string()
        });

        let final_task = super::enqueue_build_task::<ExampleBuildJobGather>(build_job_api, data_set, schema_set, ExampleBuildJobGatherInput {
            asset_id: input.asset_id,
            scatter_tasks: vec![task_id1, task_id2, task_id3]
        });

        println!("ExampleBuildJobTopLevel");
        ExampleBuildJobTopLevelOutput {
            final_task
        }
    }
}

//
// Example Scatter Job Impl
//
#[derive(Hash, Serialize, Deserialize, TypeUuid)]
#[uuid = "122248a9-9350-4ad7-8ef9-ac3795c08511"]
pub struct ExampleBuildJobScatterInput {
    pub asset_id: ObjectId,
    pub some_other_parameter: String,
}
impl BuildJobInput for ExampleBuildJobScatterInput {}

#[derive(Serialize, Deserialize)]
pub struct ExampleBuildJobScatterOutput;
impl BuildJobOutput for ExampleBuildJobScatterOutput {}

#[derive(TypeUuid)]
#[uuid = "29755562-5298-4908-8384-7b13b2cedf26"]
pub struct ExampleBuildJobScatter;

impl BuildJobWithInput for ExampleBuildJobScatter {
    type InputT = ExampleBuildJobScatterInput;
    type OutputT = ExampleBuildJobScatterOutput;

    fn enumerate_dependencies(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> BuildJobRunDependencies {
        // No dependencies
        BuildJobRunDependencies::default()
    }

    fn run(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        build_job_api: &dyn BuildJobApi
    ) -> Self::OutputT {
        //Do stuff
        // We could return the result
        // build_job_api.publish_intermediate_data(...);
        //unimplemented!();

        println!("ExampleBuildJobScatter");
        ExampleBuildJobScatterOutput {

        }
    }
}


//
// Example Gather Job Impl
//
#[derive(Hash, Serialize, Deserialize, TypeUuid)]
#[uuid = "f9b45d02-93ba-44df-8252-555f8e01d0b7"]
pub struct ExampleBuildJobGatherInput {
    pub asset_id: ObjectId,
    pub scatter_tasks: Vec<Uuid>,
}
impl BuildJobInput for ExampleBuildJobGatherInput {}

#[derive(Serialize, Deserialize)]
pub struct ExampleBuildJobGatherOutput;
impl BuildJobOutput for ExampleBuildJobGatherOutput {}

#[derive(TypeUuid)]
#[uuid = "e5f3de94-2bb6-43a9-bea0-cc91467cdcc3"]
pub struct ExampleBuildJobGather;

impl BuildJobWithInput for ExampleBuildJobGather {
    type InputT = ExampleBuildJobGatherInput;
    type OutputT = ExampleBuildJobGatherOutput;

    fn enumerate_dependencies(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> BuildJobRunDependencies {
        BuildJobRunDependencies {
            import_data: Default::default(),
            build_jobs: input.scatter_tasks.clone(),
        }
    }

    fn run(
        &self,
        input: &Self::InputT,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>,
        build_job_api: &dyn BuildJobApi
    ) -> Self::OutputT {
        // Now use inputs from other jobs to produce an output
        //build_job_api.publish_built_asset(...);

        println!("ExampleBuildJobGather");
        ExampleBuildJobGatherOutput {

        }
    }
}