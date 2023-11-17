use serde::{de, ser};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// ID of a built piece of data. A build job can produce any number of artifacts, usually reading
/// import data and/or asset data. Different platforms may have different artifacts for the same
/// conceptual piece of data
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
pub struct ArtifactId(pub Uuid);
impl ArtifactId {
    pub const fn null() -> Self {
        ArtifactId(Uuid::nil())
    }

    pub fn parse_str(input: &str) -> Result<Self, uuid::Error> {
        Ok(ArtifactId(Uuid::parse_str(input)?))
    }

    pub fn is_null(&self) -> bool {
        self.0.is_nil()
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        ArtifactId(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    pub fn from_u128(u: u128) -> Self {
        Self(Uuid::from_u128(u))
    }

    pub fn as_u128(&self) -> u128 {
        self.0.as_u128()
    }

    pub fn from_bytes(bytes: uuid::Bytes) -> Self {
        ArtifactId(Uuid::from_bytes(bytes))
    }

    pub fn as_bytes(&self) -> &uuid::Bytes {
        self.0.as_bytes()
    }
}

impl fmt::Debug for ArtifactId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_tuple("ArtifactId").field(&self.0).finish()
    }
}

impl fmt::Display for ArtifactId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for ArtifactId {
    fn serialize<S: ser::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            self.0.serialize(serializer)
        }
    }
}

struct ArtifactIdVisitor;

impl<'a> de::Visitor<'a> for ArtifactIdVisitor {
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
        Uuid::from_str(s)
            .map(|id| ArtifactId(id))
            .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(s), &self))
    }
}

impl<'de> Deserialize<'de> for ArtifactId {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_string(ArtifactIdVisitor)
        } else {
            Ok(ArtifactId(Uuid::deserialize(deserializer)?))
        }
    }
}
