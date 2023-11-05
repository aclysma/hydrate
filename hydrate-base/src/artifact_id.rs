use std::fmt;
use uuid::Uuid;
use std::fmt::{Debug, Formatter};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
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


impl Serialize for ArtifactId {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(&Uuid::from_u128(self.0).to_string())
        } else {
            Uuid::from_u128(self.0).serialize(serializer)
        }
    }
}

struct AssetTypeIdVisitor;

impl<'a> Visitor<'a> for AssetTypeIdVisitor {
    type Value = ArtifactId;

    fn expecting(
        &self,
        fmt: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(fmt, "a UUID-formatted string")
    }

    fn visit_str<E: de::Error>(
        self,
        s: &str,
    ) -> Result<Self::Value, E> {
        uuid::Uuid::parse_str(s)
            .map(|id| ArtifactId(Uuid::from_bytes(*id.as_bytes()).as_u128()))
            .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(s), &self))
    }
}

impl<'de> Deserialize<'de> for ArtifactId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_string(AssetTypeIdVisitor)
        } else {
            Ok(ArtifactId(Uuid::deserialize(deserializer)?.as_u128()))
        }
    }
}
