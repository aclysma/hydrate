use std::fmt;
#[cfg(feature = "serde-1")]
use std::str::FromStr;

#[cfg(feature = "serde-1")]
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
pub use uuid;
use uuid::Uuid;

/// UUID of an asset's Rust type. Produced by [`TypeUuidDynamic::uuid`].
///
/// If using a human-readable format, serializes to a hyphenated UUID format and deserializes from
/// any format supported by the `uuid` crate. Otherwise, serializes to and from a `[u8; 16]`.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Default, Hash)]
pub struct AssetTypeId(pub [u8; 16]);

impl AsMut<[u8]> for AssetTypeId {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl AsRef<[u8]> for AssetTypeId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for AssetTypeId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        uuid::Uuid::from_bytes(self.0).fmt(f)
    }
}

#[cfg(feature = "serde-1")]
impl Serialize for AssetTypeId {
    fn serialize<S: Serializer>(
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

#[cfg(feature = "serde-1")]
struct AssetTypeIdVisitor;

#[cfg(feature = "serde-1")]
impl<'a> Visitor<'a> for AssetTypeIdVisitor {
    type Value = AssetTypeId;

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
            .map(|id| AssetTypeId(*id.as_bytes()))
            .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(s), &self))
    }
}

#[cfg(feature = "serde-1")]
impl<'de> Deserialize<'de> for AssetTypeId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_string(AssetTypeIdVisitor)
        } else {
            Ok(AssetTypeId(<[u8; 16]>::deserialize(deserializer)?))
        }
    }
}
