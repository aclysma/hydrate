use crate::SchemaLinker;

pub struct PathNode {
    // Name and parent are part of object info, so we don't have explicit fields
}

impl PathNode {
    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |_| {})
            .unwrap();
    }

    pub fn schema_name() -> &'static str {
        "PathNode"
    }
}

pub struct PathNodeRoot {
    // Name and parent are part of object info, so we don't have explicit fields
}

impl PathNodeRoot {
    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |_| {})
            .unwrap();
    }

    pub fn schema_name() -> &'static str {
        "PathNodeRoot"
    }
}
