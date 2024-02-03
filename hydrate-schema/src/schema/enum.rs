use crate::SchemaFingerprint;
use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct SchemaEnumSymbol {
    name: String,
    symbol_uuid: Uuid,
    aliases: Box<[String]>,
}

impl SchemaEnumSymbol {
    pub fn new(
        name: String,
        symbol_uuid: Uuid,
        aliases: Box<[String]>,
    ) -> Self {
        SchemaEnumSymbol {
            name,
            symbol_uuid,
            aliases,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbol_uuid(&self) -> Uuid {
        self.symbol_uuid
    }

    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }
}

#[derive(Debug)]
pub struct SchemaEnumInner {
    name: String,
    type_uuid: Uuid,
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
        type_uuid: Uuid,
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
            type_uuid,
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

    pub fn type_uuid(&self) -> Uuid {
        self.type_uuid
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
