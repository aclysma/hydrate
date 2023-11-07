pub struct EnumTypeSymbolBuilder {
    pub(crate) name: String,
    pub(crate) aliases: Vec<String>,
}

impl EnumTypeSymbolBuilder {
    pub fn add_symbol_alias(
        &mut self,
        alias: impl Into<String>,
    ) {
        self.aliases.push(alias.into());
    }
}

#[derive(Default)]
pub struct EnumTypeBuilder {
    pub(crate) aliases: Vec<String>,
    pub(crate) symbols: Vec<EnumTypeSymbolBuilder>,
}

impl EnumTypeBuilder {
    pub fn add_type_alias(
        &mut self,
        alias: impl Into<String>,
    ) {
        self.aliases.push(alias.into())
    }

    pub fn add_symbol(
        &mut self,
        name: impl Into<String>,
    ) -> &mut EnumTypeSymbolBuilder {
        self.symbols.push(EnumTypeSymbolBuilder {
            name: name.into(),
            aliases: Default::default(),
        });
        self.symbols.last_mut().unwrap()
    }
}
