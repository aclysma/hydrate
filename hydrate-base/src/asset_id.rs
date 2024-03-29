use serde::{de, ser};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// ID for a user-edited piece of data. It may have import data associated with it. Assets can be
/// thought of as a list of properties that follow a particular schema.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
pub struct AssetId(pub Uuid);
impl AssetId {
    pub const fn null() -> Self {
        AssetId(Uuid::nil())
    }

    pub fn parse_str(input: &str) -> Result<Self, uuid::Error> {
        Ok(AssetId(Uuid::parse_str(input)?))
    }

    pub fn is_null(&self) -> bool {
        self.0.is_nil()
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        AssetId(uuid)
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
        AssetId(Uuid::from_bytes(bytes))
    }

    pub fn as_bytes(&self) -> &uuid::Bytes {
        self.0.as_bytes()
    }
}

impl fmt::Debug for AssetId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_tuple("AssetId").field(&self.0).finish()
    }
}

impl fmt::Display for AssetId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for AssetId {
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
    type Value = AssetId;

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
            .map(|id| AssetId(id))
            .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(s), &self))
    }
}

impl<'de> Deserialize<'de> for AssetId {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_string(AssetIdVisitor)
        } else {
            Ok(AssetId(Uuid::deserialize(deserializer)?))
        }
    }
}
