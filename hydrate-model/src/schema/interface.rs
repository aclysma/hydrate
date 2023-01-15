use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug)]
pub struct SchemaInterfaceInner {
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
        aliases: Box<[String]>,
    ) -> Self {
        let inner = SchemaInterfaceInner { name, aliases };

        SchemaInterface {
            inner: Arc::new(inner),
        }
    }
}
