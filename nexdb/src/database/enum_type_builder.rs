
pub struct EnumTypeSymbolBuilder {
    pub(super) name: String,
    pub(super) aliases: Vec<String>,
    pub(super) value: i32,
}

impl EnumTypeSymbolBuilder {
    pub fn add_symbol_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into());
    }
}

#[derive(Default)]
pub struct EnumTypeBuilder {
    pub(super) aliases: Vec<String>,
    pub(super) symbols: Vec<EnumTypeSymbolBuilder>,
}

impl EnumTypeBuilder {
    pub fn add_type_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into())
    }

    pub fn add_symbol(&mut self, name: impl Into<String>, value: i32) -> &mut EnumTypeSymbolBuilder {
        self.symbols.push(EnumTypeSymbolBuilder {
            name: name.into(),
            aliases: Default::default(),
            value,
        });
        self.symbols.last_mut().unwrap()
    }
}


