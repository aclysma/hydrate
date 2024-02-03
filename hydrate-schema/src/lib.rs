pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
pub type HashMapValues<'a, K, V> = std::collections::hash_map::Values<'a, K, V>;
pub type HashSet<T> = std::collections::HashSet<T, ahash::RandomState>;
pub type HashSetIter<'a, T> = std::collections::hash_set::Iter<'a, T>;

use std::fmt::{Debug, Formatter};
use uuid::Uuid;

mod property_path;
pub use property_path::PropertyPath;

mod schema;
pub use schema::*;

mod schema_def;
pub use schema_def::*;

mod schema_cache;

mod error;
pub use error::{DataSetError, DataSetErrorWithBacktrace, DataSetResult};

pub use schema_cache::CachedSchemaNamedType;
pub use schema_cache::SchemaCacheSingleFile;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SchemaFingerprint(u128);
impl SchemaFingerprint {
    pub fn as_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
    }

    pub fn from_uuid(uuid: Uuid) -> SchemaFingerprint {
        SchemaFingerprint(uuid.as_u128())
    }
}

impl Debug for SchemaFingerprint {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_tuple("SchemaFingerprint")
            .field(&Uuid::from_u128(self.0))
            .finish()
    }
}
