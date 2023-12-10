use crate::SchemaFingerprint;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug)]
pub struct SchemaEnumSymbol {
    name: String,
    aliases: Box<[String]>,
}

impl SchemaEnumSymbol {
    pub fn new(
        name: String,
        aliases: Box<[String]>,
    ) -> Self {
        SchemaEnumSymbol { name, aliases }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }
}

#[derive(Debug)]
pub struct SchemaEnumInner {
    name: String,
    fingerprint: SchemaFingerprint,
    aliases: Box<[String]>,
    symbols: Box<[SchemaEnumSymbol]>,
}

#[derive(Clone, Debug)]
pub struct SchemaEnum {
    inner: Arc<SchemaEnumInner>,
}

impl Deref for SchemaEnum {
    type Target = SchemaEnumInner;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl SchemaEnum {
    pub fn new(
        name: String,
        fingerprint: SchemaFingerprint,
        aliases: Box<[String]>,
        symbols: Box<[SchemaEnumSymbol]>,
    ) -> Self {
        assert!(!symbols.is_empty());

        // Check names are unique
        for i in 0..symbols.len() {
            for j in 0..i {
                assert_ne!(symbols[i].name, symbols[j].name);
            }
        }

        let inner = SchemaEnumInner {
            name,
            fingerprint,
            aliases,
            symbols,
        };

        SchemaEnum {
            inner: Arc::new(inner),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    pub fn symbols(&self) -> &[SchemaEnumSymbol] {
        &*self.symbols
    }

    pub fn default_value(&self) -> &SchemaEnumSymbol {
        &self.symbols[0]
    }

    pub fn fingerprint(&self) -> SchemaFingerprint {
        self.fingerprint
    }
}
