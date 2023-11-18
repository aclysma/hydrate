use demo_types::mesh_adv::{MeshAdvBlendMethod, MeshAdvShadowMethod};
use hydrate_data::*;
use hydrate_model::{DataContainerRef, DataContainerRefMut, DataContainerOwned, DataSetResult};
use std::cell::RefCell;
use std::rc::Rc;

include!("generated.rs");

impl Vec3Accessor {
    pub fn set_vec3(
        &self,
        data_container: &mut DataContainerRefMut,
        value: [f32; 3],
    ) -> DataSetResult<()> {
        self.x().set(data_container, value[0])?;
        self.y().set(data_container, value[1])?;
        self.z().set(data_container, value[2])?;
        Ok(())
    }

    pub fn get_vec3(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<[f32; 3]> {
        let x = self.x().get(data_container)?;
        let y = self.y().get(data_container)?;
        let z = self.z().get(data_container)?;
        Ok([x, y, z])
    }
}

impl Vec4Accessor {
    pub fn set_vec4(
        &self,
        data_container: &mut DataContainerRefMut,
        value: [f32; 4],
    ) -> DataSetResult<()> {
        self.x().set(data_container, value[0])?;
        self.y().set(data_container, value[1])?;
        self.z().set(data_container, value[2])?;
        self.w().set(data_container, value[3])?;
        Ok(())
    }

    pub fn get_vec4(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<[f32; 4]> {
        let x = self.x().get(data_container)?;
        let y = self.y().get(data_container)?;
        let z = self.z().get(data_container)?;
        let w = self.w().get(data_container)?;
        Ok([x, y, z, w])
    }
}

impl Into<MeshAdvBlendMethod> for MeshAdvBlendMethodEnum {
    fn into(self) -> MeshAdvBlendMethod {
        match self {
            MeshAdvBlendMethodEnum::Opaque => MeshAdvBlendMethod::Opaque,
            MeshAdvBlendMethodEnum::AlphaClip => MeshAdvBlendMethod::AlphaClip,
            MeshAdvBlendMethodEnum::AlphaBlend => MeshAdvBlendMethod::AlphaBlend,
        }
    }
}

impl Into<MeshAdvShadowMethod> for MeshAdvShadowMethodEnum {
    fn into(self) -> MeshAdvShadowMethod {
        match self {
            MeshAdvShadowMethodEnum::None => MeshAdvShadowMethod::None,
            MeshAdvShadowMethodEnum::Opaque => MeshAdvShadowMethod::Opaque,
        }
    }
}
