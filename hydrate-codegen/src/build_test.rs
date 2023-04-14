use hydrate::model::{BooleanField, F32Field, F64Field, Field, I32Field, I64Field, NullableField, ObjectRefField, PropertyPath, StringField, U32Field, U64Field};

struct AllFieldsRecord(PropertyPath);

impl Field for AllFieldsRecord {
    fn new(property_path: PropertyPath) -> Self {
        AllFieldsRecord(property_path)
    }
}

impl AllFieldsRecord {
    fn reference(&self) -> ObjectRefField {
        ObjectRefField::new(self.0.push("reference"))
    }

    fn nullable_bool(&self) -> NullableField::<BooleanField> {
        NullableField::<BooleanField>::new(self.0.push("nullable_bool"))
    }

    fn nullable_vec3(&self) -> NullableField::<Vec3Record> {
        NullableField::<Vec3Record>::new(self.0.push("nullable_vec3"))
    }

    fn boolean(&self) -> BooleanField {
        BooleanField::new(self.0.push("boolean"))
    }

    fn i32(&self) -> I32Field {
        I32Field::new(self.0.push("i32"))
    }

    fn i64(&self) -> I64Field {
        I64Field::new(self.0.push("i64"))
    }

    fn u32(&self) -> U32Field {
        U32Field::new(self.0.push("u32"))
    }

    fn u64(&self) -> U64Field {
        U64Field::new(self.0.push("u64"))
    }

    fn f32(&self) -> F32Field {
        F32Field::new(self.0.push("f32"))
    }

    fn f64(&self) -> F64Field {
        F64Field::new(self.0.push("f64"))
    }

    fn string(&self) -> StringField {
        StringField::new(self.0.push("string"))
    }
}
struct Vec4Record(PropertyPath);

impl Field for Vec4Record {
    fn new(property_path: PropertyPath) -> Self {
        Vec4Record(property_path)
    }
}

impl Vec4Record {
    fn x(&self) -> F32Field {
        F32Field::new(self.0.push("x"))
    }

    fn y(&self) -> F32Field {
        F32Field::new(self.0.push("y"))
    }

    fn z(&self) -> F32Field {
        F32Field::new(self.0.push("z"))
    }

    fn w(&self) -> F32Field {
        F32Field::new(self.0.push("w"))
    }
}
struct TransformRefRecord(PropertyPath);

impl Field for TransformRefRecord {
    fn new(property_path: PropertyPath) -> Self {
        TransformRefRecord(property_path)
    }
}

impl TransformRefRecord {
    fn transform(&self) -> ObjectRefField {
        ObjectRefField::new(self.0.push("transform"))
    }
}
struct TransformRecord(PropertyPath);

impl Field for TransformRecord {
    fn new(property_path: PropertyPath) -> Self {
        TransformRecord(property_path)
    }
}

impl TransformRecord {
    fn all_fields(&self) -> AllFieldsRecord {
        AllFieldsRecord::new(self.0.push("all_fields"))
    }

    fn position(&self) -> Vec3Record {
        Vec3Record::new(self.0.push("position"))
    }

    fn rotation(&self) -> Vec4Record {
        Vec4Record::new(self.0.push("rotation"))
    }

    fn scale(&self) -> Vec3Record {
        Vec3Record::new(self.0.push("scale"))
    }
}
struct Vec3Record(PropertyPath);

impl Field for Vec3Record {
    fn new(property_path: PropertyPath) -> Self {
        Vec3Record(property_path)
    }
}

impl Vec3Record {
    fn x(&self) -> F32Field {
        F32Field::new(self.0.push("x"))
    }

    fn y(&self) -> F32Field {
        F32Field::new(self.0.push("y"))
    }

    fn z(&self) -> F32Field {
        F32Field::new(self.0.push("z"))
    }
}
