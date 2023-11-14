use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;
use std::str::FromStr;
use serde::{de, ser};

/// Identified an artifact type. These are not schema-aware and get rebuilt if the data format
/// changes. It's defined as a UUID on the struct itself.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
pub struct ArtifactTypeId(pub Uuid);
impl ArtifactTypeId {
    pub const fn null() -> Self {
        ArtifactTypeId(Uuid::nil())
    }

    pub fn parse_str(input: &str) -> Result<Self, uuid::Error> {
        Ok(ArtifactTypeId(Uuid::parse_str(input)?))
    }

    pub fn is_null(&self) -> bool {
        self.0.is_nil()
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        ArtifactTypeId(uuid)
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
        ArtifactTypeId(Uuid::from_bytes(bytes))
    }

    pub fn as_bytes(&self) -> &uuid::Bytes {
        self.0.as_bytes()
    }
}

impl fmt::Debug for ArtifactTypeId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_tuple("AssetTypeId")
            .field(&self.0)
            .finish()
    }
}

impl fmt::Display for ArtifactTypeId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for ArtifactTypeId {
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

struct AssetIdVisitor;

impl<'a> de::Visitor<'a> for AssetIdVisitor {
    type Value = ArtifactTypeId;

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
            .map(|id| ArtifactTypeId(id))
            .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(s), &self))
    }
}

impl<'de> Deserialize<'de> for ArtifactTypeId {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_string(AssetIdVisitor)
        } else {
            Ok(ArtifactTypeId(Uuid::deserialize(deserializer)?))
        }
    }
}