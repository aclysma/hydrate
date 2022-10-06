
#[derive(Default)]
pub struct FixedTypeBuilder {
    pub(super) aliases: Vec<String>,
}

impl FixedTypeBuilder {
    pub fn add_type_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into())
    }
}

