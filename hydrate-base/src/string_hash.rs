use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum StringHashContents {
    Static(&'static str),
    Runtime(Arc<String>),
    Unknown,
}

/// Store a hash and optionally the string used to create it. The string may not be available if:
///  - The hash was created directly without a string
///  - The `strip-stringhash-strings` feature is enabled in the crate
///
/// This allows for debugging ease normally but can be used in release to save memory and avoid
/// leaking data in strings
#[derive(Clone)]
pub struct StringHash {
    hash: u128,
    // The contents are a debugging aid that may be stripped
    #[cfg(not(feature = "strip-stringhash-strings"))]
    contents: StringHashContents,
}

impl fmt::Debug for StringHash {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        #[cfg(feature = "strip-stringhash-strings")]
        {
            f.debug_struct("AssetId")
                .field("hash", &format!("{:0>32x}", self.hash))
                .finish()
        }

        #[cfg(not(feature = "strip-stringhash-strings"))]
        {
            f.debug_struct("AssetId")
                .field("hash", &format!("{:0>32x}", self.hash))
                .field("contents", &self.contents)
                .finish()
        }
    }
}

impl PartialEq for StringHash {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.hash.eq(&other.hash)
    }
}

impl Eq for StringHash {}

impl Hash for StringHash {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.hash.hash(state);
    }
}

impl StringHash {
    pub const fn from_static_str(s: &'static str) -> Self {
        let hash = if s.is_empty() {
            0u128
        } else {
            const_fnv1a_hash::fnv1a_hash_str_128(s) | 1u128
        };

        #[cfg(not(feature = "strip-stringhash-strings"))]
        let contents = StringHashContents::Static(s);

        StringHash {
            hash,
            #[cfg(not(feature = "strip-stringhash-strings"))]
            contents,
        }
    }

    pub fn from_runtime_str(s: &str) -> Self {
        let hash = if s.is_empty() {
            0u128
        } else {
            const_fnv1a_hash::fnv1a_hash_str_128(s) | 1u128
        };

        #[cfg(not(feature = "strip-stringhash-strings"))]
        let contents = StringHashContents::Runtime(Arc::new(s.to_string()));

        StringHash {
            hash,
            #[cfg(not(feature = "strip-stringhash-strings"))]
            contents,
        }
    }

    pub fn from_hash(hash: u128) -> Self {
        StringHash {
            hash,
            #[cfg(not(feature = "strip-stringhash-strings"))]
            contents: StringHashContents::Unknown,
        }
    }

    pub fn hash(&self) -> u128 {
        self.hash
    }

    #[cfg(not(feature = "strip-stringhash-strings"))]
    pub fn contents(&self) -> &StringHashContents {
        &self.contents
    }
}
