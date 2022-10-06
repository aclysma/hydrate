
pub struct EnumTypeSymbolBuilder {
    pub(crate) name: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) value: i32,
}

impl EnumTypeSymbolBuilder {
    pub fn add_symbol_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into());
    }
}

#[derive(Default)]
pub struct EnumTypeBuilder {
    pub(crate) aliases: Vec<String>,
    pub(crate) symbols: Vec<EnumTypeSymbolBuilder>,
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


