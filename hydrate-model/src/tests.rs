use crate::edit_context::EditContext;
use crate::{
    AssetLocation, AssetPath, AssetSourceId, EditContextKey, NullOverride, OverrideBehavior,
    SchemaDefType, SchemaLinker, SchemaLinkerResult, SchemaSet, UndoStack, Value,
};
use hydrate_base::AssetId;
use hydrate_data::{AssetName, SchemaSetBuilder};
use hydrate_pipeline::HydrateProjectConfiguration;
use hydrate_schema::Schema::Nullable;
use std::sync::Arc;
use uuid::Uuid;

fn asset_location() -> AssetLocation {
    AssetLocation::new(AssetId::from_uuid(
        Uuid::parse_str("57460089-9e04-4cc7-ad46-54670812da56").unwrap(),
    ))
}

fn create_vec3_schema(linker: &mut SchemaLinker) -> SchemaLinkerResult<()> {
    linker.register_record_type("Vec3", Uuid::new_v4(), |builder| {
        builder.add_f32("x", Uuid::new_v4());
        builder.add_f32("y", Uuid::new_v4());
        builder.add_f32("z", Uuid::new_v4());
    })
}

fn default_project_config() -> HydrateProjectConfiguration {
    HydrateProjectConfiguration {
        schema_def_paths: vec![],
        import_data_path: Default::default(),
        build_data_path: Default::default(),
        job_data_path: Default::default(),
        id_based_asset_sources: vec![],
        path_based_asset_sources: vec![],
        source_file_locations: vec![],
        schema_codegen_jobs: vec![],
    }
}

// We want the same fingerprint out of a record as a Schema::Record(record)
#[test]
fn set_struct_values() {
    let mut linker = SchemaLinker::default();
    create_vec3_schema(&mut linker).unwrap();

    let mut schema_set_builder = SchemaSetBuilder::default();
    schema_set_builder.add_linked_types(linker).unwrap();
    let schema_set = schema_set_builder.build();

    let undo_stack = UndoStack::default();
    let project_config = default_project_config();
    let mut db = EditContext::new(
        &project_config,
        EditContextKey::default(),
        schema_set.clone(),
        &undo_stack,
    );
    let asset_location = asset_location();

    let vec3_type = schema_set
        .find_named_type("Vec3")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj = db.new_asset(&AssetName::new("obj1"), &asset_location, &vec3_type);
    assert_eq!(
        db.resolve_property(obj, "x").unwrap().as_f32().unwrap(),
        0.0
    );
    db.set_property_override(obj, "x", Some(Value::F32(10.0)))
        .unwrap();
    assert_eq!(
        db.resolve_property(obj, "x").unwrap().as_f32().unwrap(),
        10.0
    );
    db.set_property_override(obj, "y", Some(Value::F32(20.0)))
        .unwrap();
    assert_eq!(
        db.resolve_property(obj, "y").unwrap().as_f32().unwrap(),
        20.0
    );
    db.set_property_override(obj, "z", Some(Value::F32(30.0)))
        .unwrap();
    assert_eq!(
        db.resolve_property(obj, "z").unwrap().as_f32().unwrap(),
        30.0
    );
}

#[test]
fn set_struct_values_in_struct() {
    let mut linker = SchemaLinker::default();
    create_vec3_schema(&mut linker).unwrap();

    linker
        .register_record_type("OuterStruct", Uuid::new_v4(), |builder| {
            builder.add_named_type("a", Uuid::new_v4(), "Vec3");
            builder.add_named_type("b", Uuid::new_v4(), "Vec3");
        })
        .unwrap();

    let mut schema_set_builder = SchemaSetBuilder::default();
    schema_set_builder.add_linked_types(linker).unwrap();
    let schema_set = schema_set_builder.build();

    let undo_stack = UndoStack::default();
    let project_config = default_project_config();
    let mut db = EditContext::new(
        &project_config,
        EditContextKey::default(),
        schema_set.clone(),
        &undo_stack,
    );
    let asset_location = asset_location();

    //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
    let outer_struct_type = schema_set
        .find_named_type("OuterStruct")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj = db.new_asset(&AssetName::new("test"), &asset_location, &outer_struct_type);
    assert_eq!(
        db.resolve_property(obj, "a.x").unwrap().as_f32().unwrap(),
        0.0
    );
    db.set_property_override(obj, "a.x", Some(Value::F32(10.0)))
        .unwrap();
    assert_eq!(
        db.resolve_property(obj, "a.x").unwrap().as_f32().unwrap(),
        10.0
    );
    assert_eq!(
        db.resolve_property(obj, "b.x").unwrap().as_f32().unwrap(),
        0.0
    );
    db.set_property_override(obj, "b.x", Some(Value::F32(20.0)))
        .unwrap();
    assert_eq!(
        db.resolve_property(obj, "a.x").unwrap().as_f32().unwrap(),
        10.0
    );
    assert_eq!(
        db.resolve_property(obj, "b.x").unwrap().as_f32().unwrap(),
        20.0
    );
}

#[test]
fn set_simple_property_override() {
    let mut linker = SchemaLinker::default();
    create_vec3_schema(&mut linker).unwrap();

    let mut schema_set_builder = SchemaSetBuilder::default();
    schema_set_builder.add_linked_types(linker).unwrap();
    let schema_set = schema_set_builder.build();

    let undo_stack = UndoStack::default();
    let project_config = default_project_config();
    let mut db = EditContext::new(
        &project_config,
        EditContextKey::default(),
        schema_set.clone(),
        &undo_stack,
    );
    let asset_location = asset_location();

    let vec3_type = schema_set
        .find_named_type("Vec3")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj1 = db.new_asset(&AssetName::new("test"), &asset_location, &vec3_type);
    let obj2 = db
        .new_asset_from_prototype(&AssetName::new("test2"), &asset_location, obj1)
        .unwrap();
    assert_eq!(
        db.resolve_property(obj1, "x").unwrap().as_f32().unwrap(),
        0.0
    );
    assert_eq!(
        db.resolve_property(obj2, "x").unwrap().as_f32().unwrap(),
        0.0
    );
    assert_eq!(db.has_property_override(obj1, "x").unwrap(), false);
    assert_eq!(db.has_property_override(obj2, "x").unwrap(), false);
    assert_eq!(db.get_property_override(obj1, "x").unwrap().is_none(), true);
    assert_eq!(db.get_property_override(obj2, "x").unwrap().is_none(), true);

    db.set_property_override(obj1, "x", Some(Value::F32(10.0)))
        .unwrap();
    assert_eq!(
        db.resolve_property(obj1, "x").unwrap().as_f32().unwrap(),
        10.0
    );
    assert_eq!(
        db.resolve_property(obj2, "x").unwrap().as_f32().unwrap(),
        10.0
    );
    assert_eq!(db.has_property_override(obj1, "x").unwrap(), true);
    assert_eq!(db.has_property_override(obj2, "x").unwrap(), false);
    assert_eq!(
        db.get_property_override(obj1, "x")
            .unwrap()
            .unwrap()
            .as_f32()
            .unwrap(),
        10.0
    );
    assert_eq!(db.get_property_override(obj2, "x").unwrap().is_none(), true);

    db.set_property_override(obj2, "x", Some(Value::F32(20.0)));
    assert_eq!(
        db.resolve_property(obj1, "x").unwrap().as_f32().unwrap(),
        10.0
    );
    assert_eq!(
        db.resolve_property(obj2, "x").unwrap().as_f32().unwrap(),
        20.0
    );
    assert_eq!(db.has_property_override(obj1, "x").unwrap(), true);
    assert_eq!(db.has_property_override(obj2, "x").unwrap(), true);
    assert_eq!(
        db.get_property_override(obj1, "x")
            .unwrap()
            .unwrap()
            .as_f32()
            .unwrap(),
        10.0
    );
    assert_eq!(
        db.get_property_override(obj2, "x")
            .unwrap()
            .unwrap()
            .as_f32()
            .unwrap(),
        20.0
    );

    db.set_property_override(obj1, "x", None).unwrap();
    assert_eq!(
        db.resolve_property(obj1, "x").unwrap().as_f32().unwrap(),
        0.0
    );
    assert_eq!(
        db.resolve_property(obj2, "x").unwrap().as_f32().unwrap(),
        20.0
    );
    assert_eq!(db.has_property_override(obj1, "x").unwrap(), false);
    assert_eq!(db.has_property_override(obj2, "x").unwrap(), true);
    assert_eq!(db.get_property_override(obj1, "x").unwrap().is_none(), true);
    assert_eq!(
        db.get_property_override(obj2, "x")
            .unwrap()
            .unwrap()
            .as_f32()
            .unwrap(),
        20.0
    );

    db.set_property_override(obj2, "x", None).unwrap();
    assert_eq!(
        db.resolve_property(obj1, "x").unwrap().as_f32().unwrap(),
        0.0
    );
    assert_eq!(
        db.resolve_property(obj2, "x").unwrap().as_f32().unwrap(),
        0.0
    );
    assert_eq!(db.has_property_override(obj1, "x").unwrap(), false);
    assert_eq!(db.has_property_override(obj2, "x").unwrap(), false);
    assert_eq!(db.get_property_override(obj1, "x").unwrap().is_none(), true);
    assert_eq!(db.get_property_override(obj2, "x").unwrap().is_none(), true);
}

// Tests below this point rotted

/*
#[test]
fn property_in_nullable() {
    // let vec3_schema_record = create_vec3_schema();
    //
    // let outer_struct = SchemaRecord::new("OuterStruct".to_string(), vec![].into_boxed_slice(), vec![
    //     SchemaRecordField::new(
    //         "nullable".to_string(),
    //         vec![].into_boxed_slice(),
    //         Schema::Nullable(Box::new(Schema::Record(vec3_schema_record)))
    //     )
    // ].into_boxed_slice());
    //
    // let mut db = Database::default();

    let mut linker = SchemaLinker::default();
    create_vec3_schema(&mut linker).unwrap();

    linker
        .register_record_type("OuterStruct", Uuid::new_v4(), |builder| {
            builder.add_nullable("nullable", Uuid::new_v4(), SchemaDefType::NamedType("Vec3".to_string()));
        })
        .unwrap();

    let mut schema_set_builder = SchemaSetBuilder::default();
    schema_set_builder.add_linked_types(linker).unwrap();
    let schema_set = schema_set_builder.build();

    let undo_stack = UndoStack::default();
    let project_config = default_project_config();
    let mut db = EditContext::new(&project_config, EditContextKey::default(), schema_set.clone(), &undo_stack);
    let asset_location = asset_location();

    let outer_struct_type = schema_set
        .find_named_type("OuterStruct")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj = db.new_asset(&AssetName::new("test"), &asset_location, &outer_struct_type);

    assert_eq!(db.resolve_null_override(obj, "nullable").unwrap(), NullOverride::Unset);
    assert!(
        db.resolve_property(obj, "nullable.value.x")
            .unwrap().as_nullable().unwrap().is_none()
    );
    // This should fail because we are trying to set a null value
    assert!(db.set_property_override(obj, "nullable.value.x", Some(Value::F32(10.0))).is_err());
    assert_eq!(db.resolve_null_override(obj, "nullable").unwrap(), NullOverride::Unset);
    assert!(
        db.resolve_property(obj, "nullable.value.x")
            .unwrap().as_nullable().unwrap().is_none(),
    );
    /*
    db.set_null_override(obj, "nullable", NullOverride::SetNonNull);
    assert_eq!(db.resolve_null_override(obj, "nullable").unwrap(), false);
    assert_eq!(db.resolve_null_override(obj, "nullable"), Some(false));
    // This is still set to 0 because the above set should have failed
    assert_eq!(
        db.resolve_property(obj, "nullable.value.x")
            .unwrap().as_f32().unwrap(),
        Some(0.0)
    );
    db.set_property_override(obj, "nullable.value.x", Value::F32(10.0));
    assert_eq!(
        db.resolve_property(obj, "nullable.value.x")
            .unwrap().as_f32().unwrap(),
        Some(10.0)
    );

     */
}

#[test]
fn nullable_property_in_nullable() {
    // let vec3_schema_record = create_vec3_schema();
    //
    // let outer_struct = SchemaRecord::new("OuterStruct".to_string(), vec![].into_boxed_slice(), vec![
    //     SchemaRecordField::new(
    //         "nullable".to_string(),
    //         vec![].into_boxed_slice(),
    //         Schema::Nullable(Box::new(Schema::Nullable(Box::new(Schema::Record(vec3_schema_record)))))
    //     )
    // ].into_boxed_slice());
    //
    // let mut db = Database::default();

    let mut linker = SchemaLinker::default();
    create_vec3_schema(&mut linker).unwrap();

    linker
        .register_record_type("OuterStruct", Uuid::new_v4(), |builder| {
            builder.add_nullable(
                "nullable",
                Uuid::new_v4(),
                SchemaDefType::Nullable(Box::new(SchemaDefType::NamedType("Vec3".to_string()))),
            );
        })
        .unwrap();

    let mut schema_set_builder = SchemaSetBuilder::default();
    schema_set_builder.add_linked_types(linker).unwrap();
    let schema_set = schema_set_builder.build();

    let undo_stack = UndoStack::default();
    let project_config = default_project_config();
    let mut db = EditContext::new(&project_config, EditContextKey::default(), schema_set.clone(), &undo_stack);
    let asset_location = asset_location();

    //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
    let outer_struct_type = schema_set
        .find_named_type("OuterStruct")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj = db.new_asset(&AssetName::new("test"), &asset_location, &outer_struct_type);

    assert_eq!(db.resolve_null_override(obj, "nullable").unwrap(), true);
    // This returns none because parent property is null, so this property should act like it doesn't exist
    assert_eq!(db.resolve_null_override(obj, "nullable.value"), None);
    assert_eq!(
        db.resolve_property(obj, "nullable.value.value.x")
            .unwrap().as_f32().unwrap(),
        None
    );
    // This attempt to set should fail because an ancestor path is null
    assert!(!db.set_property_override(obj, "nullable.value.value.x", Value::F32(10.0)));
    assert_eq!(db.resolve_null_override(obj, "nullable").unwrap(), true);
    assert_eq!(db.resolve_null_override(obj, "nullable.value"), None);
    assert_eq!(
        db.resolve_property(obj, "nullable.value.value.x")
            .unwrap().as_f32().unwrap(),
        None
    );
    db.set_null_override(obj, "nullable", NullOverride::SetNonNull);
    assert_eq!(db.resolve_null_override(obj, "nullable").unwrap(), false);
    assert_eq!(
        db.resolve_null_override(obj, "nullable.value").unwrap(),
        true
    );
    assert_eq!(
        db.resolve_property(obj, "nullable.value.value.x")
            .unwrap().as_f32().unwrap(),
        None
    );
    db.set_null_override(obj, "nullable.value", NullOverride::SetNonNull);
    assert_eq!(db.resolve_null_override(obj, "nullable").unwrap(), false);
    assert_eq!(
        db.resolve_null_override(obj, "nullable.value").unwrap(),
        false
    );
    // This is default value because the attempt to set it to 10 above should have failed
    assert_eq!(
        db.resolve_property(obj, "nullable.value.value.x")
            .unwrap().as_f32().unwrap(),
        Some(0.0)
    );
    assert!(db.set_property_override(obj, "nullable.value.value.x", Value::F32(10.0)));
    assert_eq!(
        db.resolve_property(obj, "nullable.value.value.x")
            .unwrap().as_f32().unwrap(),
        Some(10.0)
    );
}

//TODO: Test override nullable

#[test]
fn struct_in_dynamic_array() {
    // let vec3_schema_record = create_vec3_schema();
    //
    // let outer_struct = SchemaRecord::new("OuterStruct".to_string(), vec![].into_boxed_slice(), vec![
    //     SchemaRecordField::new(
    //         "array".to_string(),
    //         vec![].into_boxed_slice(),
    //         Schema::DynamicArray(SchemaDynamicArray::new(Box::new(Schema::Record(vec3_schema_record))))
    //     )
    // ].into_boxed_slice());
    //
    // let mut db = Database::default();

    let mut linker = SchemaLinker::default();
    create_vec3_schema(&mut linker).unwrap();

    linker
        .register_record_type("OuterStruct", Uuid::new_v4(), |builder| {
            builder.add_dynamic_array("array", Uuid::new_v4(), SchemaDefType::NamedType("Vec3".to_string()));
        })
        .unwrap();

    let mut schema_set_builder = SchemaSetBuilder::default();
    schema_set_builder.add_linked_types(linker).unwrap();
    let schema_set = schema_set_builder.build();

    let undo_stack = UndoStack::default();
    let project_config = default_project_config();
    let mut db = EditContext::new(&project_config, EditContextKey::default(), schema_set.clone(), &undo_stack);
    let asset_location = asset_location();

    //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
    let outer_struct_type = schema_set
        .find_named_type("OuterStruct")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj = db.new_asset(&AssetName::new("test"), &asset_location, &outer_struct_type);

    assert!(db.resolve_dynamic_array_entries(obj, "array").is_empty());
    let uuid1 = db.add_dynamic_array_entry(obj, "array");
    let prop1 = format!("array.{}.x", uuid1);
    assert_eq!(
        db.resolve_dynamic_array_entries(obj, "array"),
        vec![uuid1].into_boxed_slice()
    );
    let uuid2 = db.add_dynamic_array_entry(obj, "array");
    let prop2 = format!("array.{}.x", uuid2);
    let resolved = db.resolve_dynamic_array_entries(obj, "array");
    assert!(resolved.contains(&uuid1));
    assert!(resolved.contains(&uuid2));

    assert_eq!(
        db.resolve_property(obj, &prop1).unwrap().as_f32().unwrap(),
        0.0
    );
    assert_eq!(
        db.resolve_property(obj, &prop2).unwrap().as_f32().unwrap(),
        0.0
    );
    db.set_property_override(obj, &prop1, Value::F32(10.0));
    assert_eq!(
        db.resolve_property(obj, &prop1).unwrap().as_f32().unwrap(),
        10.0
    );
    assert_eq!(
        db.resolve_property(obj, &prop2).unwrap().as_f32().unwrap(),
        0.0
    );
    db.set_property_override(obj, &prop2, Value::F32(20.0));
    assert_eq!(
        db.resolve_property(obj, &prop1).unwrap().as_f32().unwrap(),
        10.0
    );
    assert_eq!(
        db.resolve_property(obj, &prop2).unwrap().as_f32().unwrap(),
        20.0
    );

    db.remove_dynamic_array_entry(obj, "array", uuid1);
    assert_eq!(
        db.resolve_dynamic_array_entries(obj, "array"),
        vec![uuid2].into_boxed_slice()
    );
    assert!(db.resolve_property(obj, &prop1).is_none());
    assert_eq!(
        db.resolve_property(obj, &prop2).unwrap().as_f32().unwrap(),
        20.0
    );
}

#[test]
fn dynamic_array_override_behavior() {
    // let vec3_schema_record = create_vec3_schema();
    //
    // let outer_struct = SchemaRecord::new("OuterStruct".to_string(), vec![].into_boxed_slice(), vec![
    //     SchemaRecordField::new(
    //         "array".to_string(),
    //         vec![].into_boxed_slice(),
    //         Schema::DynamicArray(SchemaDynamicArray::new(Box::new(Schema::Record(vec3_schema_record))))
    //     )
    // ].into_boxed_slice());
    //
    // let mut db = Database::default();

    let mut linker = SchemaLinker::default();
    create_vec3_schema(&mut linker).unwrap();

    linker
        .register_record_type("OuterStruct", Uuid::new_v4(), |builder| {
            builder.add_dynamic_array("array", Uuid::new_v4(), SchemaDefType::NamedType("Vec3".to_string()));
        })
        .unwrap();

    let mut schema_set_builder = SchemaSetBuilder::default();
    schema_set_builder.add_linked_types(linker).unwrap();
    let schema_set = schema_set_builder.build();

    let undo_stack = UndoStack::default();
    let project_config = default_project_config();
    let mut db = EditContext::new(&project_config, EditContextKey::default(), schema_set.clone(), &undo_stack);
    let asset_location = asset_location();

    //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
    let outer_struct_type = schema_set
        .find_named_type("OuterStruct")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj1 = db.new_asset(&AssetName::new("test"), &asset_location, &outer_struct_type);
    let obj2 = db.new_asset_from_prototype(&AssetName::new("test2"), &asset_location, obj1);

    let item1 = db.add_dynamic_array_entry(obj1, "array");
    let item2 = db.add_dynamic_array_entry(obj2, "array");

    assert_eq!(
        db.resolve_dynamic_array_entries(obj1, "array"),
        vec![item1].into_boxed_slice()
    );
    assert_eq!(
        db.resolve_dynamic_array_entries(obj2, "array"),
        vec![item1, item2].into_boxed_slice()
    );

    // This should fail, this override is on obj2, not obj1
    assert!(!db.set_property_override(obj1, format!("array.{}.x", item2), Value::F32(20.0)));

    db.set_property_override(obj1, format!("array.{}.x", item1), Value::F32(10.0));
    db.set_property_override(obj2, format!("array.{}.x", item2), Value::F32(20.0));

    db.set_override_behavior(obj2, "array", OverrideBehavior::Replace);
    assert_eq!(
        db.resolve_dynamic_array_entries(obj2, "array"),
        vec![item2].into_boxed_slice()
    );

    assert!(db
        .resolve_property(obj2, format!("array.{}.x", item1))
        .is_none());
    assert_eq!(
        db.resolve_property(obj2, format!("array.{}.x", item2))
            .unwrap()
            .as_f32()
            .unwrap(),
        20.0
    );

    // This should fail, this override is on obj1 which we no longer inherit
    assert!(!db.set_property_override(obj2, format!("array.{}.x", item1), Value::F32(30.0)));

    db.set_override_behavior(obj2, "array", OverrideBehavior::Append);
    assert_eq!(
        db.resolve_dynamic_array_entries(obj2, "array"),
        vec![item1, item2].into_boxed_slice()
    );
}


 */
