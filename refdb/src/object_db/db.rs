use super::*;

#[derive(Default)]
pub struct ObjectDb {
    interface_types: Vec<InterfaceType>,
    object_types: Vec<ObjectType>,
    objects: SlotMap<ObjectKey, ObjectInfo>,
    type_by_name: AHashMap<String, TypeId>,
    type_by_uuid: AHashMap<Uuid, TypeId>,
}

impl ObjectDb {
    pub fn inteface_type(&self, id: InterfaceTypeId) -> &InterfaceType {
        &self.interface_types[id.0 as usize]
    }

    fn interface_type_mut(&mut self, id: InterfaceTypeId) -> &mut InterfaceType {
        &mut self.interface_types[id.0 as usize]
    }

    pub fn object_type(&self, id: ObjectTypeId) -> &ObjectType {
        &self.object_types[id.0 as usize]
    }

    fn object_type_mut(&mut self, id: ObjectTypeId) -> &mut ObjectType {
        &mut self.object_types[id.0 as usize]
    }

    pub fn register_interface_type<S: Into<String>>(
        &mut self,
        uuid: uuid::Uuid,
        name: S,
    ) -> ObjectDbResult<InterfaceTypeId> {
        //TODO: Check name not empty
        //TODO: Check name/uuid not duplicated? Otherwise lookups won't work
        //TODO: Hash name
        //TODO: Return existing type if already registered

        if self.object_types.len() >= MAX_INTERFACE_TYPE_COUNT {
            Err(format!("More than {} interface types not supported", MAX_INTERFACE_TYPE_COUNT))?;
        }

        // Create the type
        let name = name.into();
        let interface_type = InterfaceType {
            name: name.clone(),
            implementors: Default::default()
        };

        // Add the type to the list of types and appropriate lookups
        let type_id = InterfaceTypeId(self.interface_types.len() as u8);
        self.interface_types.push(interface_type);
        let old = self.type_by_name.insert(name, TypeId::Interface(type_id));
        assert!(old.is_none());
        let old = self.type_by_uuid.insert(uuid, TypeId::Interface(type_id));
        assert!(old.is_none());

        Ok(type_id)
    }

    pub fn register_object_type<S: Into<String>>(
        &mut self,
        uuid: uuid::Uuid,
        name: S,
        properties: &[PropertyDef]
    ) -> ObjectDbResult<ObjectTypeId> {
        //TODO: Check name not empty
        //TODO: Check name not duplicated? Otherwise lookups won't work
        //TODO: Hash name
        //TODO: Return existing type if already registered

        if properties.len() > MAX_PROPERTY_COUNT {
            Err(format!("More than {} properties not supported", MAX_PROPERTY_COUNT))?;
        }

        if self.object_types.len() >= MAX_OBJECT_TYPE_COUNT {
            Err(format!("More than {} object types not supported", MAX_OBJECT_TYPE_COUNT))?;
        }

        for p in properties {
            // A null default value for a concrete subobject type is interpreted as the default object of the concrete type
            // So if it's an interface subobject, verify not null. (We don't know what concrete type to use by default,
            // so there's no obvious default value we can use)
            let mut allow_null_subobject = false;
            if let PropertyType::Subobject(ty) = p.property_type {
                allow_null_subobject = ty.is_concrete();
            }

            if !p.default_value.is_type(self, p.property_type, allow_null_subobject) {
                Err(format!("The given value {:?} cannot be used as a default value for property {} of type {:?} (allow_null_subobject={})", p.default_value, p.name, p.property_type, allow_null_subobject))?;
            }
        }

        // Create the type
        let name = name.into();
        let properties : Vec<PropertyDef> = properties.iter().cloned().collect();
        let object_type = ObjectType {
            name: name.clone(),
            properties,
            interfaces: Default::default()
        };

        // Add the type to the list of types and appropriate lookups
        let type_id = ObjectTypeId(self.object_types.len() as u16);
        self.object_types.push(object_type);
        let old = self.type_by_name.insert(name, TypeId::Object(type_id));
        assert!(old.is_none());
        let old = self.type_by_uuid.insert(uuid, TypeId::Object(type_id));
        assert!(old.is_none());

        Ok(type_id)
    }

    pub fn add_implementor(&mut self, interface: InterfaceTypeId, implementor: ObjectTypeId) {
        self.interface_type_mut(interface).implementors.insert(implementor);
        self.object_type_mut(implementor).interfaces.set(interface.0 as usize, true);
    }

    pub fn find_type_by_name(&self, name: &str) -> Option<TypeId> {
        self.type_by_name.get(name).copied()
    }

    pub fn find_type_by_uuid(&self, uuid: &Uuid) -> Option<TypeId> {
        self.type_by_uuid.get(uuid).copied()
    }

    pub fn type_id_of_object(&self, object_id: ObjectId) -> ObjectTypeId {
        let object = &self.objects[object_id.0];
        object.object_type_id
    }

    pub fn is_object_type_allowed(&self, object_type_id: ObjectTypeId, selector: TypeSelector) -> bool {
        match selector {
            TypeSelector::Any => true,
            TypeSelector::Interface(type_id) => {
                self.object_type(object_type_id).interfaces.is_set(type_id.0 as usize)
            },
            TypeSelector::Object(type_id) => {
                object_type_id == type_id
            }
        }
    }

    // Detached properties are copies. Attached implies a prototype instance with a field that is
    // not overridden
    fn create_property_value(&mut self, p: PropertyType, v: Value, detached: bool) -> Value {
        match p {
            // For subobjects:
            // - If the value is null, create a default object.
            //   * It is only allowed to be null if the type is concrete. (If it's not concrete, we
            //     won't know the intended default concrete type to use.)
            //   * If it's null, we know we are creating a new object (not a prototype instance.) Prototype
            //     instances can only be created from existing objects, and subobject properties of
            //     existing objects can't be null.)
            // - If the value is non-null, clone the object, or create a prototype instance of it
            PropertyType::Subobject(ty) => {
                let subobject = v.get_subobject().unwrap();
                if subobject.is_null() {
                    // If it's null, we *must* be creating an entirely new object that has no
                    // prototype.
                    assert!(detached);

                    match ty {
                        TypeSelector::Object(object_type) => {
                            let object_id = self.create_object(object_type);
                            Value::Subobject(object_id.0)
                        },
                        _ => panic!("A subobject with non-concrete type selector cannot have a null default value")
                    }
                } else {
                    let prototype = ObjectId(subobject);
                    let new_subobject = if detached {
                        self.clone_object(prototype)
                    } else {
                        self.create_prototype_instance(prototype)
                    };

                    Value::Subobject(new_subobject.0)
                }
            },
            _ => v
        }
    }

    pub fn create_object(&mut self, type_id: ObjectTypeId) -> ObjectId {
        let property_count = self.object_type(type_id).properties.len();
        let mut property_values = Vec::<Value>::with_capacity(property_count);
        for property_index in 0..property_count {
            let p = &self.object_type(type_id).properties[property_index];
            property_values.push(self.create_property_value(p.property_type, p.default_value.clone(), true));
        }

        let object_id = self.objects.insert(ObjectInfo {
            prototype: ObjectKey::null(),
            object_type_id: type_id,
            property_values,
            inherited_properties: PropertyBits::default(),
        });

        ObjectId(object_id)
    }

    pub fn create_prototype_instance(&mut self, prototype_object_id: ObjectId) -> ObjectId {
        debug_assert!(!prototype_object_id.0.is_null());
        let prototype = &self.objects[prototype_object_id.0];
        let object_type_id = prototype.object_type_id;
        let object_type = self.object_type(object_type_id);

        let property_count = object_type.properties.len();
        let mut property_values = Vec::<Value>::with_capacity(property_count);
        for property_index in 0..property_count {
            let p = &self.object_type(object_type_id).properties[property_index];
            let property_value = self.property_value(prototype_object_id, PropertyIndex(property_index as u8)).clone();
            property_values.push(self.create_property_value(p.property_type, property_value, false));
        }

        let mut inherited_properties = PropertyBits::default();
        inherited_properties.set_first_n(property_count, true);
        let object_id = self.objects.insert(ObjectInfo {
            prototype: prototype_object_id.0,
            object_type_id: object_type_id,
            property_values,
            inherited_properties,
        });

        ObjectId(object_id)
    }

    pub fn detach_from_prototype(&mut self, object_id: ObjectId) {
        let object = &mut self.objects[object_id.0];

        if object.prototype.is_null() {
            // No-op, we are not attached to any prototype
            return;
        }

        // Clear the prototype
        let prototype = object.prototype;
        object.prototype = ObjectKey::null();

        // Clear inherited properties
        let inherited_properties = object.inherited_properties;
        object.inherited_properties = PropertyBits::default();

        // Copy any inherited value from the prototype into this object.
        let property_count = object.property_values.len();
        for i in 0..property_count {
            if inherited_properties.is_set(i) {
                self.objects[object_id.0].property_values[i] = self.objects[prototype].property_values[i].clone();
            }
        }
    }

    pub fn clone_object(&mut self, object_to_clone: ObjectId) -> ObjectId {
        let mut object = self.create_prototype_instance(object_to_clone);
        self.detach_from_prototype(object);
        object
    }

    //pub fn copy_object()

    pub fn find_property(&self, type_id: ObjectTypeId, name: &str) -> Option<PropertyIndex> {
        let p = self.object_type(type_id).properties.iter().position(|x| x.name == name);
        p.map(|x| PropertyIndex(x as u8))
    }

    // pub fn value(&self, object_id: ObjectId, property: PropertyIndex) -> Value {
    //     self.objects[object_id.0].property_values[property.0 as usize]
    // }
    //
    // pub fn value_mut(&mut self, object_id: ObjectId, property: PropertyIndex) -> &mut Value {
    //     &mut self.objects[object_id.0].property_values[property.0 as usize]
    // }

    fn property_value(&self, mut object_id: ObjectId, property: PropertyIndex) -> &Value {
        let object = &self.objects[object_id.0];
        let property_type = self.object_type(object.object_type_id).properties[property.0 as usize].property_type;
        if !property_type.is_primitive_value() {
            return &object.property_values[property.0 as usize];
        }

        let mut object_key = object_id.0;
        loop {
            if object_key.is_null() {
                // Use the type's default value
                break;
            }

            let object = &self.objects[object_key];
            if !object.inherited_properties.is_set(property.0 as usize) {
                // Use this object's value
                break;
            } else {
                // Continue looping, checking the next prototype up the tree
                object_key = object.prototype;
            }
        }

        if object_key.is_null() {
            let object_type_id = self.objects[object_id.0].object_type_id;
            &self.object_type(object_type_id).properties[property.0 as usize].default_value
        } else {
            &self.objects[object_key].property_values[property.0 as usize]
        }
    }

    pub fn get_u64(&self, object_id: ObjectId, property: PropertyIndex) -> ObjectDbResult<u64> {
        self.property_value(object_id, property).get_u64()
    }

    pub fn set_u64(&mut self, object_id: ObjectId, property: PropertyIndex, value: u64) -> ObjectDbResult<()> {
        let object = &mut self.objects[object_id.0];
        object.property_values[property.0 as usize].set_u64(value)?;
        object.inherited_properties.set(property.0 as usize, false);
        Ok(())
    }

    pub fn get_f32(&self, object_id: ObjectId, property: PropertyIndex) -> ObjectDbResult<f32> {
        self.property_value(object_id, property).get_f32()
    }

    pub fn set_f32(&mut self, object_id: ObjectId, property: PropertyIndex, value: f32) -> ObjectDbResult<()> {
        let object = &mut self.objects[object_id.0];
        object.property_values[property.0 as usize].set_f32(value)?;
        object.inherited_properties.set(property.0 as usize, false);
        Ok(())
    }

    pub fn get_subobject(&self, object_id: ObjectId, property: PropertyIndex) -> ObjectDbResult<ObjectId> {
        self.property_value(object_id, property).get_subobject().map(|x| ObjectId(x))
    }

    // pub fn set_subobject(&mut self, object_id: ObjectId, property: PropertyIndex, subobject_id: ObjectId) -> ObjectDbResult<()> {
    //     let object_type_id = self.type_id_of_object(object_id);
    //     let object_type = self.object_type(object_type_id);
    //     let type_selector = object_type.properties[property.0 as usize].property_type.type_selector().ok_or(ObjectDbError::TypeError)?;
    //     let subobject_type_id = self.type_id_of_object(subobject_id);
    //     if !self.is_object_type_allowed(subobject_type_id, type_selector) {
    //         Err(ObjectDbError::TypeError)?;
    //     }
    //
    //     let object = &mut self.objects[object_id.0];
    //     object.property_values[property.0 as usize].set_subobject(subobject_id.0)?;
    //     object.inherited_properties.set(property.0 as usize, false);
    //     Ok(())
    // }

    pub fn clear_property_override(&mut self, object_id: ObjectId, property: PropertyIndex) {
        let object = &mut self.objects[object_id.0];
        object.inherited_properties.set(property.0 as usize, true);
    }

    pub fn set_property_override(&mut self, object_id: ObjectId, property: PropertyIndex) {
        let current_value = self.property_value(object_id, property).clone();
        let object = &mut self.objects[object_id.0];
        // We can get away without cloning the subobject here because the property will be flagged as
        // inherited, which means we won't use it directly. (it's essentially a move, not a copy)
        object.property_values[property.0 as usize] = current_value;
        object.inherited_properties.set(property.0 as usize, false);
    }

    //TODO: Improve API to handle a chain of prototypes?
    pub fn apply_property_override_to_prototype(&mut self, object_id: ObjectId, property: PropertyIndex) -> ObjectDbResult<()> {
        let current_value = self.property_value(object_id, property).clone();
        let prototype = self.objects[object_id.0].prototype;
        assert!(!prototype.is_null());

        let prototype = &mut self.objects[prototype];
        prototype.property_values[property.0 as usize] = current_value;
        prototype.inherited_properties.set(property.0 as usize, false);

        let object = &mut self.objects[object_id.0];
        object.inherited_properties.set(property.0 as usize, true);
        Ok(())
    }

    pub fn is_property_inherited(&mut self, object_id: ObjectId, property: PropertyIndex) -> bool {
        let object = &mut self.objects[object_id.0];
        object.inherited_properties.is_set(property.0 as usize)
    }




    // Allocator settings

    // Buffer Management

    // String Management

    // register_type
    // set_default_object/set_default_values?
    // get_default_object
    // is_default
    //
    // register/applying interfaces
    //
    // find/enumerate types
    // find/enumerate properties
    // find/enumerate interfaces
    //
    // find objects (type, filtering, etc.)
    //
    // undo/redo
    //
    // creating/cloning objects
    // adding/removing subobjects
    //
    // garbage collect deleted?
    //
    // get/set properties, subobjects
    //
    // save
    //
    // apply_to_base_prefab
    // detach from prototype
    //
    // get_base_prefab
    // check_if_overridden
    //
    // change detection
    //


    //
}