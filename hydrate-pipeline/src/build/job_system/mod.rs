mod job_system_traits;
pub use job_system_traits::*;

mod executor;
pub use executor::*;

mod job_executor_thread_pool;
use job_executor_thread_pool::*;

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

crate::create_uuid_newtype!(JobId, "JobId");
crate::create_uuid_newtype!(JobTypeId, "JobTypeId");

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct JobVersion(u32);
