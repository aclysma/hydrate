use hydrate::model::{DataSet, DataSetView, F32Field, ObjectId, PropertyPath};

struct TransformRefFromSchema {
    transform: ObjectId,
}
impl TransformRefFromSchema {
    pub fn load(data_set_view: &mut DataSetView) -> Self {
        let transform = data_set_view.resolve_property("transform").unwrap().as_object_ref().unwrap();

        Self {
            transform,
        }
    }
}
struct AllFieldsFromSchema {
    reference: ObjectId,
    nullable_bool: Option<bool>,
    nullable_vec3: Option<Vec3FromSchema>,
    boolean: bool,
    i32: i32,
    i64: i64,
    u32: u32,
    u64: u64,
    f32: f32,
    f64: f64,
    string: String,
    dynamic_array_i32: Vec<i32>,
    dynamic_array_vec3: Vec<Vec3FromSchema>,
}
impl AllFieldsFromSchema {
    pub fn load(data_set_view: &mut DataSetView) -> Self {
        let reference = data_set_view.resolve_property("reference").unwrap().as_object_ref().unwrap();
        let nullable_bool = if !data_set_view.resolve_is_null("nullable_bool").unwrap() {
            data_set_view.push_property_path("nullable_bool");
            let value = data_set_view.resolve_property("value").unwrap().as_boolean().unwrap();
            data_set_view.pop_property_path();
            Some(value)
        } else {
            None
        };
        let nullable_vec3 = if !data_set_view.resolve_is_null("nullable_vec3").unwrap() {
            data_set_view.push_property_path("nullable_vec3");
            let value = {
                data_set_view.push_property_path("value");
                let value = Vec3FromSchema::load(data_set_view);
                data_set_view.pop_property_path();
                value
            };
            data_set_view.pop_property_path();
            Some(value)
        } else {
            None
        };
        let boolean = data_set_view.resolve_property("boolean").unwrap().as_boolean().unwrap();
        let i32 = data_set_view.resolve_property("i32").unwrap().as_i32().unwrap();
        let i64 = data_set_view.resolve_property("i64").unwrap().as_i64().unwrap();
        let u32 = data_set_view.resolve_property("u32").unwrap().as_u32().unwrap();
        let u64 = data_set_view.resolve_property("u64").unwrap().as_u64().unwrap();
        let f32 = data_set_view.resolve_property("f32").unwrap().as_f32().unwrap();
        let f64 = data_set_view.resolve_property("f64").unwrap().as_f64().unwrap();
        let string = data_set_view.resolve_property("string").unwrap().as_string().unwrap();
        let dynamic_array_i32 = {
        };
        let dynamic_array_vec3 = {
        };

        Self {
            reference,
            nullable_bool,
            nullable_vec3,
            boolean,
            i32,
            i64,
            u32,
            u64,
            f32,
            f64,
            string,
            dynamic_array_i32,
            dynamic_array_vec3,
        }
    }
}


// Ability to build type-safe paths
// Ability to fetch fields into native rust structure
// Ability to fetch fields, mutate some of them, and store just the changes as overrides (generate diff?)
// - having actual references has problems with supporting hot reloads
//


struct V4 {
    x: f32, y: f32, z: f32, w: f32
}

struct MyV4 {
    x: f32, y: f32, z: f32, w: f32
}

trait ToVec4 {
    fn to_vec4(&self) -> MyV4;
}

impl ToVec4 for V4 {
    fn to_vec4(&self) -> MyV4 {
        MyV4 { x: self.x, y: self.y, z: self.z, w: self.w }
    }
}

fn try_it_out() {
    let v4 = V4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0};
    let my_v4 = v4.to_vec4();
}


// what if this is a property builder?

// #[derive(Default)]
// struct PropertyPath(String);
//
// impl PropertyPath {
//     pub fn push(&self, str: &str) -> PropertyPath {
//         PropertyPath(format!("{}.{}", self.0, str))
//     }
//
//     pub fn path(&self) -> &str {
//         &self.0
//     }
// }




//struct PropertyPathWalker(Vec<String>);

#[derive(Default)]
struct Vec4Record(PropertyPath);

impl Vec4Record {
    pub fn x(&self) -> F32Field {
        F32Field(self.0.push("x"))
    }
    pub fn y(&self) -> F32Field {
        F32Field(self.0.push("y"))
    }
    pub fn z(&self) -> F32Field {
        F32Field(self.0.push("z"))
    }
    pub fn w(&self) -> F32Field {
        F32Field(self.0.push("w"))
    }

    pub fn get(&self, data_set_view: &mut DataSetView) -> Vec4FromSchema {
        data_set_view.push_property_path(self.0.path());
        let v = Vec4FromSchema::load();
        data_set_view.pop_property_path();
        v
    }
}

#[derive(Default)]
struct TransformRecord(PropertyPath);

impl TransformRecord {
    pub fn position(&self) -> Vec4PW {
        Vec4PW(self.0.push("position"))
    }
}





fn try_it(data_set_view: &DataSetView) {

    //TransformPW::
    //TransformPW::default().position().

    //data_set_view.


    //let position_path = TransformPW::default().position();
    // let x = position_path.x().get(data_set_view);
    // let y = position_path.y().get(data_set_view);
    // let z = position_path.z().get(data_set_view);
}


struct Vec4FromSchema2<'a>(&'a mut DataSetView<'a>);

impl Vec4FromSchema2 {
    pub fn x(&self) -> f32 {
        data_set_view.resolve_property("x").unwrap().as_f32().unwrap()
    }

    pub fn y(&self) -> f32 {
        data_set_view.resolve_property("x").unwrap().as_f32().unwrap()
    }
}

struct TransformFromSchema2<'a>(&'a mut DataSetView<'a>);

impl<'a> TransformFromSchema2<'a> {
    pub fn position(&self) -> Vec4FromSchema2<'a> {
        let x = self.0.clone().push_property_path("position");
        Vec4FromSchema2(&x)
    }
}



// struct TransformFromSchema {
//     all_fields: AllFieldsFromSchema,
//     position: Vec3FromSchema,
//     rotation: Vec4FromSchema,
//     scale: Vec3FromSchema,
// }



struct Vec4FromSchema {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}
impl Vec4FromSchema {
    pub fn load(data_set_view: &mut DataSetView) -> Self {
        let x = data_set_view.resolve_property("x").unwrap().as_f32().unwrap();
        let y = data_set_view.resolve_property("y").unwrap().as_f32().unwrap();
        let z = data_set_view.resolve_property("z").unwrap().as_f32().unwrap();
        let w = data_set_view.resolve_property("w").unwrap().as_f32().unwrap();

        Self {
            x,
            y,
            z,
            w,
        }
    }
}
struct Vec3FromSchema {
    x: f32,
    y: f32,
    z: f32,
}
impl Vec3FromSchema {
    pub fn load(data_set_view: &mut DataSetView) -> Self {
        let x = data_set_view.resolve_property("x").unwrap().as_f32().unwrap();
        let y = data_set_view.resolve_property("y").unwrap().as_f32().unwrap();
        let z = data_set_view.resolve_property("z").unwrap().as_f32().unwrap();

        Self {
            x,
            y,
            z,
        }
    }
}

struct TransformFromSchema {
    all_fields: AllFieldsFromSchema,
    position: Vec3FromSchema,
    rotation: Vec4FromSchema,
    scale: Vec3FromSchema,
}

impl TransformFromSchema {
    pub fn load(data_set_view: &mut DataSetView) -> Self {
        let all_fields = {
            data_set_view.push_property_path("all_fields");
            let value = AllFieldsFromSchema::load(data_set_view);
            data_set_view.pop_property_path();
            value
        };
        let position = {
            data_set_view.push_property_path("position");
            let value = Vec3FromSchema::load(data_set_view);
            data_set_view.pop_property_path();
            value
        };
        let rotation = {
            data_set_view.push_property_path("rotation");
            let value = Vec4FromSchema::load(data_set_view);
            data_set_view.pop_property_path();
            value
        };
        let scale = {
            data_set_view.push_property_path("scale");
            let value = Vec3FromSchema::load(data_set_view);
            data_set_view.pop_property_path();
            value
        };

        Self {
            all_fields,
            position,
            rotation,
            scale,
        }
    }
}