use crate::SchemaLinker;
use uuid::Uuid;

pub struct PathNode {
    // Name and parent are part of asset info, so we don't have explicit fields
}

impl PathNode {
    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(
                Self::schema_name(),
                Uuid::parse_str("ec66c632-fed2-40fc-b27a-c0c138ad31bc").unwrap(),
                |_| {},
            )
            .unwrap();
    }

    pub fn schema_name() -> &'static str {
        "PathNode"
    }
}

pub struct PathNodeRoot {
    // Name and parent are part of asset info, so we don't have explicit fields
}

impl PathNodeRoot {
    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(
                Self::schema_name(),
                Uuid::parse_str("da09e646-89a5-41d3-8029-048ed1ad1b3b").unwrap(),
                |_| {},
            )
            .unwrap();
    }

    pub fn schema_name() -> &'static str {
        "PathNodeRoot"
    }
}
