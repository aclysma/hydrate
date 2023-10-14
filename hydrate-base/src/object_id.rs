
use uuid::Uuid;
use std::fmt::{Debug, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ObjectId(pub u128);
impl ObjectId {
    pub const fn null() -> Self {
        ObjectId(0)
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        ObjectId(uuid.as_u128())
    }

    pub fn as_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
    }

    pub fn is_null(&self) -> bool {
        return self.0 == 0;
    }
}

impl Debug for ObjectId {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_tuple("ObjectId")
            .field(&Uuid::from_u128(self.0))
            .finish()
    }
}
