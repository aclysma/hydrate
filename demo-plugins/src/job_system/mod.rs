mod traits;
mod executor;
mod example_tasks;

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

use traits::*;
use executor::*;
use example_tasks::*;



#[test]
fn test_builders() {
    let mut executor = BuildJobExecutor::default();
    executor.register_job_type(ExampleBuildJobTopLevel);
    executor.register_job_type(ExampleBuildJobGather);
    executor.register_job_type(ExampleBuildJobScatter);

    let data_set = DataSet::default();
    let schema_set = SchemaSet::default();

    let job_id = enqueue_build_task::<ExampleBuildJobTopLevel>(&executor, &data_set, &schema_set, ExampleBuildJobTopLevelInput {
        asset_id: ObjectId::null(),
    });

    while !executor.is_idle() {
        executor.update(&data_set, &schema_set);
    }

    // Get results
}
