
use uuid::Uuid;
use std::fmt::{Debug, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ArtifactId(pub u128);
impl ArtifactId {
    pub const fn null() -> Self {
        ArtifactId(0)
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        ArtifactId(uuid.as_u128())
    }

    pub fn as_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
    }

    pub fn is_null(&self) -> bool {
        return self.0 == 0;
    }

    pub fn from_u128(u: u128) -> Self {
        Self(u)
    }

    pub fn as_u128(&self) -> u128 {
        self.0
    }
}

impl Debug for ArtifactId {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_tuple("ArtifactId")
            .field(&Uuid::from_u128(self.0))
            .finish()
    }
}
