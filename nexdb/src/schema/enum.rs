use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;
use crate::schema::SchemaTypeIndex;

#[derive(Debug)]
pub struct SchemaEnumSymbol {
    name: String,
    aliases: Box<[String]>,
    value: i32
}

impl SchemaEnumSymbol {
    pub(crate) fn fingerprint_hash<T: Hasher>(&self, hasher: &mut T) {
        self.name.hash(hasher);
        self.value.hash(hasher);
    }
}

impl SchemaEnumSymbol {
    pub fn new(name: String, aliases: Box<[String]>, value: i32) -> Self {
        SchemaEnumSymbol {
            name,
            aliases,
            value
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

#[derive(Debug)]
pub struct SchemaEnumInner {
    name: String,
    aliases: Box<[String]>,
    symbols: Box<[SchemaEnumSymbol]>
}

#[derive(Clone, Debug)]
pub struct SchemaEnum {
    inner: Arc<SchemaEnumInner>
}

impl Deref for SchemaEnum {
    type Target = SchemaEnumInner;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl SchemaEnum {
    pub fn new(name: String, aliases: Box<[String]>, symbols: Box<[SchemaEnumSymbol]>) -> Self {
        // Check symbols are sorted
        for i in 0..symbols.len() - 1 {
            assert!(symbols[i].value < symbols[i + 1].value);
        }

        // Check names are unique
        for i in 0..symbols.len() {
            for j in 0..i {
                assert_ne!(symbols[i].name, symbols[j].name);
            }
        }

        let inner = SchemaEnumInner {
            name,
            aliases,
            symbols
        };

        SchemaEnum {
            inner: Arc::new(inner)
        }
    }

    pub(crate) fn fingerprint_hash<T: Hasher>(&self, hasher: &mut T) {
        SchemaTypeIndex::Enum.fingerprint_hash(hasher);
        self.inner.name.hash(hasher);
        for symbol in self.inner.symbols.iter() {
            symbol.fingerprint_hash(hasher);
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbols(&self) -> &[SchemaEnumSymbol] {
        &*self.symbols
    }
}
