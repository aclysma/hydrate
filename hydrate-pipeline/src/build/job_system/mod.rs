mod job_system_traits;
pub use job_system_traits::*;

mod executor;
pub use executor::*;

mod job_executor_thread_pool;
use job_executor_thread_pool::*;

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use uuid::Uuid;

macro_rules! create_uuid_newtype {
    ($data_type:ident, $name:literal) => {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
        pub struct $data_type(u128);
        impl $data_type {
            pub const fn null() -> Self {
                Self(0)
            }

            pub fn from_bytes(bytes: [u8; 16]) -> Self {
                Self(Uuid::from_bytes(bytes).as_u128())
            }

            pub fn as_bytes(self) -> [u8; 16] {
                *Uuid::from_u128(self.0).as_bytes()
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid.as_u128())
            }

            pub fn as_uuid(self) -> Uuid {
                Uuid::from_u128(self.0)
            }

            pub fn from_u128(u: u128) -> Self {
                Self(u)
            }

            pub fn as_u128(self) -> u128 {
                self.0
            }

            pub fn is_null(&self) -> bool {
                return self.0 == 0;
            }
        }

        impl Debug for $data_type {
            fn fmt(
                &self,
                f: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                f.debug_tuple($name)
                    .field(&Uuid::from_u128(self.0))
                    .finish()
            }
        }
    };
}

create_uuid_newtype!(JobId, "JobId");
create_uuid_newtype!(JobTypeId, "JobTypeId");
create_uuid_newtype!(JobHash, "JobTypeId");

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct JobVersion(u32);
