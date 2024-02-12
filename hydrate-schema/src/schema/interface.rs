// This is not used yet

/*
use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct SchemaInterfaceInner {
    type_uuid: Uuid,
    name: String,
    aliases: Box<[String]>,
}

#[derive(Clone, Debug)]
pub struct SchemaInterface {
    inner: Arc<SchemaInterfaceInner>,
}

impl Deref for SchemaInterface {
    type Target = SchemaInterfaceInner;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl SchemaInterface {
    pub fn new(
        name: String,
        type_uuid: Uuid,
        aliases: Box<[String]>,
    ) -> Self {
        let inner = SchemaInterfaceInner { name, type_uuid, aliases };

        SchemaInterface {
            inner: Arc::new(inner),
        }
    }
}
*/