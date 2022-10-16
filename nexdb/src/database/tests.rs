use crate::{
    Database, NullOverride, OverrideBehavior, Schema, SchemaDefDynamicArray, SchemaDefType,
    SchemaDynamicArray, SchemaLinker, SchemaLinkerResult, SchemaRecord, SchemaRecordField, Value,
};

fn create_vec3_schema(linker: &mut SchemaLinker) -> SchemaLinkerResult<()> {
    linker.register_record_type("Vec3", |builder| {
        builder.add_f32("x");
        builder.add_f32("y");
        builder.add_f32("z");
    })
}

// We want the same fingerprint out of a record as a Schema::Record(record)
#[test]
fn set_struct_values() {
    let mut linker = SchemaLinker::default();
    create_vec3_schema(&mut linker).unwrap();

    let mut db = Database::default();
    db.add_linked_types(linker).unwrap();
    let vec3_type = db
        .find_named_type("Vec3")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj = db.new_object(&vec3_type);
    assert_eq!(
        db.resolve_property(obj, "x").map(|x| x.as_f32()),
        Some(Some(0.0))
    );
    db.set_property_override(obj, "x", Value::F32(10.0));
    assert_eq!(
        db.resolve_property(obj, "x").map(|x| x.as_f32()),
        Some(Some(10.0))
    );
    db.set_property_override(obj, "y", Value::F32(20.0));
    assert_eq!(
        db.resolve_property(obj, "y").map(|x| x.as_f32()),
        Some(Some(20.0))
    );
    db.set_property_override(obj, "z", Value::F32(30.0));
    assert_eq!(
        db.resolve_property(obj, "z").map(|x| x.as_f32()),
        Some(Some(30.0))
    );
}

#[test]
fn set_struct_values_in_struct() {
    let mut linker = SchemaLinker::default();
    create_vec3_schema(&mut linker).unwrap();

    linker
        .register_record_type("OuterStruct", |builder| {
            builder.add_struct("a", "Vec3");
            builder.add_struct("b", "Vec3");
        })
        .unwrap();

    let mut db = Database::default();
    db.add_linked_types(linker).unwrap();
    //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
    let outer_struct_type = db
        .find_named_type("OuterStruct")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj = db.new_object(&outer_struct_type);
    assert_eq!(
        db.resolve_property(obj, "a.x").map(|x| x.as_f32()),
        Some(Some(0.0))
    );
    db.set_property_override(obj, "a.x", Value::F32(10.0));
    assert_eq!(
        db.resolve_property(obj, "a.x").map(|x| x.as_f32()),
        Some(Some(10.0))
    );
    assert_eq!(
        db.resolve_property(obj, "b.x").map(|x| x.as_f32()),
        Some(Some(0.0))
    );
    db.set_property_override(obj, "b.x", Value::F32(20.0));
    assert_eq!(
        db.resolve_property(obj, "a.x").map(|x| x.as_f32()),
        Some(Some(10.0))
    );
    assert_eq!(
        db.resolve_property(obj, "b.x").map(|x| x.as_f32()),
        Some(Some(20.0))
    );
}

#[test]
fn set_simple_property_override() {
    let mut linker = SchemaLinker::default();
    create_vec3_schema(&mut linker).unwrap();

    let mut db = Database::default();
    db.add_linked_types(linker).unwrap();
    let vec3_type = db
        .find_named_type("Vec3")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj1 = db.new_object(&vec3_type);
    let obj2 = db.new_object_from_prototype(obj1);
    assert_eq!(
        db.resolve_property(obj1, "x").map(|x| x.as_f32().unwrap()),
        Some(0.0)
    );
    assert_eq!(
        db.resolve_property(obj2, "x").map(|x| x.as_f32().unwrap()),
        Some(0.0)
    );
    assert_eq!(db.has_property_override(obj1, "x"), false);
    assert_eq!(db.has_property_override(obj2, "x"), false);
    assert_eq!(db.get_property_override(obj1, "x").is_none(), true);
    assert_eq!(db.get_property_override(obj2, "x").is_none(), true);

    db.set_property_override(obj1, "x", Value::F32(10.0));
    assert_eq!(
        db.resolve_property(obj1, "x").map(|x| x.as_f32().unwrap()),
        Some(10.0)
    );
    assert_eq!(
        db.resolve_property(obj2, "x").map(|x| x.as_f32().unwrap()),
        Some(10.0)
    );
    assert_eq!(db.has_property_override(obj1, "x"), true);
    assert_eq!(db.has_property_override(obj2, "x"), false);
    assert_eq!(
        db.get_property_override(obj1, "x")
            .unwrap()
            .as_f32()
            .unwrap(),
        10.0
    );
    assert_eq!(db.get_property_override(obj2, "x").is_none(), true);

    db.set_property_override(obj2, "x", Value::F32(20.0));
    assert_eq!(
        db.resolve_property(obj1, "x").map(|x| x.as_f32().unwrap()),
        Some(10.0)
    );
    assert_eq!(
        db.resolve_property(obj2, "x").map(|x| x.as_f32().unwrap()),
        Some(20.0)
    );
    assert_eq!(db.has_property_override(obj1, "x"), true);
    assert_eq!(db.has_property_override(obj2, "x"), true);
    assert_eq!(
        db.get_property_override(obj1, "x")
            .unwrap()
            .as_f32()
            .unwrap(),
        10.0
    );
    assert_eq!(
        db.get_property_override(obj2, "x")
            .unwrap()
            .as_f32()
            .unwrap(),
        20.0
    );

    db.remove_property_override(obj1, "x");
    assert_eq!(
        db.resolve_property(obj1, "x").map(|x| x.as_f32().unwrap()),
        Some(0.0)
    );
    assert_eq!(
        db.resolve_property(obj2, "x").map(|x| x.as_f32().unwrap()),
        Some(20.0)
    );
    assert_eq!(db.has_property_override(obj1, "x"), false);
    assert_eq!(db.has_property_override(obj2, "x"), true);
    assert_eq!(db.get_property_override(obj1, "x").is_none(), true);
    assert_eq!(
        db.get_property_override(obj2, "x")
            .unwrap()
            .as_f32()
            .unwrap(),
        20.0
    );

    db.remove_property_override(obj2, "x");
    assert_eq!(
        db.resolve_property(obj1, "x").map(|x| x.as_f32().unwrap()),
        Some(0.0)
    );
    assert_eq!(
        db.resolve_property(obj2, "x").map(|x| x.as_f32().unwrap()),
        Some(0.0)
    );
    assert_eq!(db.has_property_override(obj1, "x"), false);
    assert_eq!(db.has_property_override(obj2, "x"), false);
    assert_eq!(db.get_property_override(obj1, "x").is_none(), true);
    assert_eq!(db.get_property_override(obj2, "x").is_none(), true);
}

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
        .register_record_type("OuterStruct", |builder| {
            builder.add_nullable("nullable", SchemaDefType::NamedType("Vec3".to_string()));
        })
        .unwrap();

    let mut db = Database::default();
    db.add_linked_types(linker).unwrap();
    //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap().clone();
    let outer_struct_type = db
        .find_named_type("OuterStruct")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj = db.new_object(&outer_struct_type);

    assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), true);
    assert_eq!(
        db.resolve_property(obj, "nullable.value.x")
            .map(|x| x.as_f32().unwrap()),
        None
    );
    // This should fail because we are trying to set a null value
    assert!(!db.set_property_override(obj, "nullable.value.x", Value::F32(10.0)));
    assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), true);
    assert_eq!(
        db.resolve_property(obj, "nullable.value.x")
            .map(|x| x.as_f32().unwrap()),
        None
    );
    db.set_null_override(obj, "nullable", NullOverride::SetNonNull);
    assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), false);
    assert_eq!(db.resolve_is_null(obj, "nullable"), Some(false));
    // This is still set to 0 because the above set should have failed
    assert_eq!(
        db.resolve_property(obj, "nullable.value.x")
            .map(|x| x.as_f32().unwrap()),
        Some(0.0)
    );
    db.set_property_override(obj, "nullable.value.x", Value::F32(10.0));
    assert_eq!(
        db.resolve_property(obj, "nullable.value.x")
            .map(|x| x.as_f32().unwrap()),
        Some(10.0)
    );
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
        .register_record_type("OuterStruct", |builder| {
            builder.add_nullable(
                "nullable",
                SchemaDefType::Nullable(Box::new(SchemaDefType::NamedType("Vec3".to_string()))),
            );
        })
        .unwrap();

    let mut db = Database::default();
    db.add_linked_types(linker).unwrap();
    //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
    let outer_struct_type = db
        .find_named_type("OuterStruct")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj = db.new_object(&outer_struct_type);

    assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), true);
    // This returns none because parent property is null, so this property should act like it doesn't exist
    assert_eq!(db.resolve_is_null(obj, "nullable.value"), None);
    assert_eq!(
        db.resolve_property(obj, "nullable.value.value.x")
            .map(|x| x.as_f32().unwrap()),
        None
    );
    // This attempt to set should fail because an ancestor path is null
    assert!(!db.set_property_override(obj, "nullable.value.value.x", Value::F32(10.0)));
    assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), true);
    assert_eq!(db.resolve_is_null(obj, "nullable.value"), None);
    assert_eq!(
        db.resolve_property(obj, "nullable.value.value.x")
            .map(|x| x.as_f32().unwrap()),
        None
    );
    db.set_null_override(obj, "nullable", NullOverride::SetNonNull);
    assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), false);
    assert_eq!(db.resolve_is_null(obj, "nullable.value").unwrap(), true);
    assert_eq!(
        db.resolve_property(obj, "nullable.value.value.x")
            .map(|x| x.as_f32().unwrap()),
        None
    );
    db.set_null_override(obj, "nullable.value", NullOverride::SetNonNull);
    assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), false);
    assert_eq!(db.resolve_is_null(obj, "nullable.value").unwrap(), false);
    // This is default value because the attempt to set it to 10 above should have failed
    assert_eq!(
        db.resolve_property(obj, "nullable.value.value.x")
            .map(|x| x.as_f32().unwrap()),
        Some(0.0)
    );
    assert!(db.set_property_override(obj, "nullable.value.value.x", Value::F32(10.0)));
    assert_eq!(
        db.resolve_property(obj, "nullable.value.value.x")
            .map(|x| x.as_f32().unwrap()),
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
        .register_record_type("OuterStruct", |builder| {
            builder.add_dynamic_array("array", SchemaDefType::NamedType("Vec3".to_string()));
        })
        .unwrap();

    let mut db = Database::default();
    db.add_linked_types(linker).unwrap();
    //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
    let outer_struct_type = db
        .find_named_type("OuterStruct")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj = db.new_object(&outer_struct_type);

    assert!(db.resolve_dynamic_array(obj, "array").is_empty());
    let uuid1 = db.add_dynamic_array_override(obj, "array");
    let prop1 = format!("array.{}.x", uuid1);
    assert_eq!(
        db.resolve_dynamic_array(obj, "array"),
        vec![uuid1].into_boxed_slice()
    );
    let uuid2 = db.add_dynamic_array_override(obj, "array");
    let prop2 = format!("array.{}.x", uuid2);
    let resolved = db.resolve_dynamic_array(obj, "array");
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

    db.remove_dynamic_array_override(obj, "array", uuid1);
    assert_eq!(
        db.resolve_dynamic_array(obj, "array"),
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
        .register_record_type("OuterStruct", |builder| {
            builder.add_dynamic_array("array", SchemaDefType::NamedType("Vec3".to_string()));
        })
        .unwrap();

    let mut db = Database::default();
    db.add_linked_types(linker).unwrap();
    //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
    let outer_struct_type = db
        .find_named_type("OuterStruct")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    let obj1 = db.new_object(&outer_struct_type);
    let obj2 = db.new_object_from_prototype(obj1);

    let item1 = db.add_dynamic_array_override(obj1, "array");
    let item2 = db.add_dynamic_array_override(obj2, "array");

    assert_eq!(
        db.resolve_dynamic_array(obj1, "array"),
        vec![item1].into_boxed_slice()
    );
    assert_eq!(
        db.resolve_dynamic_array(obj2, "array"),
        vec![item1, item2].into_boxed_slice()
    );

    // This should fail, this override is on obj2, not obj1
    assert!(!db.set_property_override(obj1, format!("array.{}.x", item2), Value::F32(20.0)));

    db.set_property_override(obj1, format!("array.{}.x", item1), Value::F32(10.0));
    db.set_property_override(obj2, format!("array.{}.x", item2), Value::F32(20.0));

    db.set_override_behavior(obj2, "array", OverrideBehavior::Replace);
    assert_eq!(
        db.resolve_dynamic_array(obj2, "array"),
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
        db.resolve_dynamic_array(obj2, "array"),
        vec![item1, item2].into_boxed_slice()
    );
}
