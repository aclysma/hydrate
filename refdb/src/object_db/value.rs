
pub use super::*;

#[derive(Clone, Debug)]
pub enum Value {
    U64(u64),
    F32(f32),
    // Ref(ObjectKey),
    Subobject(ObjectKey),
    // RefSet(AHashSet<ObjectKey>),
    // SubobjectSet(AHashSet<ObjectKey>),
}

impl Value {
    pub fn can_convert_to(&self, db: &ObjectDb, property_type: PropertyType) -> bool {
        match self {
            Value::U64(_) => property_type == PropertyType::U64,
            Value::F32(_) => property_type == PropertyType::F32,
            Value::Subobject(o) => {
                if let PropertyType::Subobject(type_selector) = property_type {
                    let object_type_id = db.type_id_of_object(ObjectId(*o));
                    db.is_object_type_allowed(object_type_id, type_selector)
                } else {
                    false
                }
            }
            // Value::Reference(_) => unimplemented!(),
            // Value::Subobject(_) => unimplemented!(),
            // Value::ReferenceSet(_) => unimplemented!(),
            // Value::SubobjectSet(_) => unimplemented!(),
        }
    }

    pub fn convert_to(self, property_type: PropertyType) -> Option<Self> {
        match property_type {
            PropertyType::U64 => self.convert_to_u64(),
            PropertyType::F32 => self.convert_to_f32(),
            PropertyType::Subobject(_) => self.convert_to_subobject(),
        }
    }

    pub fn convert_to_u64(self) -> Option<Self> {
        Some(match self {
            Value::U64(_) => self,
            //Value::F32(v) => Value::U64(v as u64),
            _ => return None,
        })
    }

    pub fn convert_to_f32(self) -> Option<Self> {
        Some(match self {
            Value::F32(_) => self,
            //Value::U64(v) => Value::F32(v as f32),
            _ => return None,
        })
    }

    pub fn convert_to_subobject(self) -> Option<Self> {
        Some(match self {
            //Value::F32(v) => Value::F32(v),
            //Value::U64(v) => Value::F32(v as f32),
            Value::Subobject(v) => self,
            _ => return None,
        })
    }

    pub(super) fn get_u64(&self) -> ObjectDbResult<u64> {
        match self {
            Value::U64(v) => Ok(*v),
            _ => Err(ObjectDbError::TypeError)
        }
    }

    pub(super) fn set_u64(&mut self, value: u64) -> ObjectDbResult<()> {
        match self {
            Value::U64(v) => {
                *v = value;
                Ok(())
            },
            _ => Err(ObjectDbError::TypeError)
        }
    }

    pub(super) fn get_f32(&self) -> ObjectDbResult<f32> {
        match self {
            Value::F32(v) => Ok(*v),
            _ => Err(ObjectDbError::TypeError)
        }
    }

    pub(super) fn set_f32(&mut self, value: f32) -> ObjectDbResult<()> {
        match self {
            Value::F32(v) => {
                *v = value;
                Ok(())
            },
            _ => Err(ObjectDbError::TypeError)
        }
    }

    pub(super) fn get_subobject(&self) -> ObjectDbResult<ObjectKey> {
        match self {
            Value::Subobject(v) => Ok(*v),
            _ => Err(ObjectDbError::TypeError)
        }
    }

    pub(super) fn set_subobject(&mut self, value: ObjectKey) -> ObjectDbResult<()> {
        match self {
            Value::Subobject(v) => {
                *v = value;
                Ok(())
            },
            _ => Err(ObjectDbError::TypeError)
        }
    }
}
