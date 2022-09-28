use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;
use crate::schema::SchemaTypeIndex;

#[derive(Debug)]
pub struct SchemaFixedInner {
    name: String,
    aliases: Box<[String]>,
    length: usize,
}

#[derive(Clone, Debug)]
pub struct SchemaFixed {
    inner: Arc<SchemaFixedInner>
}

impl Deref for SchemaFixed {
    type Target = SchemaFixedInner;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl SchemaFixed {
    pub fn new(name: String, aliases: Box<[String]>, length: usize) -> Self {
        let inner = SchemaFixedInner {
            name,
            aliases,
            length
        };

        SchemaFixed {
            inner: Arc::new(inner)
        }
    }

    pub(crate) fn fingerprint_hash<T: Hasher>(&self, hasher: &mut T) {
        SchemaTypeIndex::Fixed.fingerprint_hash(hasher);
        self.name.hash(hasher);
        self.length.hash(hasher);
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
