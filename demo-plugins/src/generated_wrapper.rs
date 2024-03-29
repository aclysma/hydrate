use demo_types::mesh_adv::{MeshAdvBlendMethod, MeshAdvShadowMethod};
use hydrate_data::*;
use hydrate_model::{DataContainer, DataContainerRef, DataContainerRefMut, DataSetResult};
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
        let x = self.x().get(data_container.clone())?;
        let y = self.y().get(data_container.clone())?;
        let z = self.z().get(data_container.clone())?;
        Ok([x, y, z])
    }
}

impl<'a> Vec3Ref<'a> {
    pub fn get_vec3(&self) -> DataSetResult<[f32; 3]> {
        let x = self.x().get()?;
        let y = self.y().get()?;
        let z = self.z().get()?;
        Ok([x, y, z])
    }
}

impl Vec3Record {
    pub fn set_vec3(
        &self,
        value: [f32; 3],
    ) -> DataSetResult<()> {
        self.x().set(value[0])?;
        self.y().set(value[1])?;
        self.z().set(value[2])?;
        Ok(())
    }

    pub fn get_vec3(&self) -> DataSetResult<[f32; 3]> {
        let x = self.x().get()?;
        let y = self.y().get()?;
        let z = self.z().get()?;
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
        let x = self.x().get(data_container.clone())?;
        let y = self.y().get(data_container.clone())?;
        let z = self.z().get(data_container.clone())?;
        let w = self.w().get(data_container.clone())?;
        Ok([x, y, z, w])
    }
}

impl<'a> Vec4Ref<'a> {
    pub fn get_vec4(&self) -> DataSetResult<[f32; 4]> {
        let x = self.x().get()?;
        let y = self.y().get()?;
        let z = self.z().get()?;
        let w = self.w().get()?;
        Ok([x, y, z, w])
    }
}

impl Vec4Record {
    pub fn set_vec4(
        &self,
        value: [f32; 4],
    ) -> DataSetResult<()> {
        self.x().set(value[0])?;
        self.y().set(value[1])?;
        self.z().set(value[2])?;
        self.w().set(value[3])?;
        Ok(())
    }

    pub fn get_vec4(&self) -> DataSetResult<[f32; 4]> {
        let x = self.x().get()?;
        let y = self.y().get()?;
        let z = self.z().get()?;
        let w = self.w().get()?;
        Ok([x, y, z, w])
    }
}

impl ColorRgbaU8Accessor {
    pub fn set_vec4(
        &self,
        data_container: &mut DataContainerRefMut,
        value: [f32; 4],
    ) -> DataSetResult<()> {
        self.r().set(data_container, (value[0] * 255.0) as u32)?;
        self.g().set(data_container, (value[1] * 255.0) as u32)?;
        self.b().set(data_container, (value[2] * 255.0) as u32)?;
        self.a().set(data_container, (value[3] * 255.0) as u32)?;
        Ok(())
    }

    pub fn get_vec4(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<[f32; 4]> {
        let r = self.r().get(data_container.clone())?;
        let g = self.g().get(data_container.clone())?;
        let b = self.b().get(data_container.clone())?;
        let a = self.a().get(data_container.clone())?;
        Ok([r as f32, g as f32, b as f32, a as f32])
    }
}

impl<'a> ColorRgbaU8Ref<'a> {
    pub fn get_vec4(&self) -> DataSetResult<[f32; 4]> {
        let r = self.r().get()?;
        let g = self.g().get()?;
        let b = self.b().get()?;
        let a = self.a().get()?;
        Ok([r as f32, g as f32, b as f32, a as f32])
    }
}

impl ColorRgbaU8Record {
    pub fn set_vec4(
        &self,
        value: [f32; 4],
    ) -> DataSetResult<()> {
        self.r().set((value[0] * 255.0) as u32)?;
        self.g().set((value[1] * 255.0) as u32)?;
        self.b().set((value[2] * 255.0) as u32)?;
        self.a().set((value[3] * 255.0) as u32)?;
        Ok(())
    }

    pub fn get_vec4(&self) -> DataSetResult<[f32; 4]> {
        let r = self.r().get()?;
        let g = self.g().get()?;
        let b = self.b().get()?;
        let a = self.a().get()?;
        Ok([r as f32, g as f32, b as f32, a as f32])
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
