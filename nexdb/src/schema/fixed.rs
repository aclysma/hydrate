use crate::SchemaFingerprint;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug)]
pub struct SchemaFixedInner {
    name: String,
    fingerprint: SchemaFingerprint,
    aliases: Box<[String]>,
    length: usize,
}

#[derive(Clone, Debug)]
pub struct SchemaFixed {
    inner: Arc<SchemaFixedInner>,
}

impl Deref for SchemaFixed {
    type Target = SchemaFixedInner;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl SchemaFixed {
    pub fn new(
        name: String,
        fingerprint: SchemaFingerprint,
        aliases: Box<[String]>,
        length: usize,
    ) -> Self {
        let inner = SchemaFixedInner {
            name,
            fingerprint,
            aliases,
            length,
        };

        SchemaFixed {
            inner: Arc::new(inner),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn aliases(&self) -> &[String] {
        &*self.aliases
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn fingerprint(&self) -> SchemaFingerprint {
        self.fingerprint
    }
}
