
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::ObjectId;

#[derive(Debug, Serialize, Deserialize)]
pub struct BuiltObjectMetadata {
    pub dependencies: Vec<ObjectId>,
    pub subresource_count: u32,
    pub asset_type: Uuid, // size?
}

impl BuiltObjectMetadata {
    pub fn write_header<T: std::io::Write>(
        &self,
        writer: &mut T,
    ) -> std::io::Result<()> {
        // writer.write(&(self.dependencies.len() as u32).to_le_bytes())?;
        // for dependency in &self.dependencies {
        //     writer.write(&dependency.0.to_le_bytes())?;
        // }
        //
        // writer.write(&self.subresource_count.to_le_bytes())?;
        // writer.write(&self.asset_type.as_u128().to_le_bytes())?;

        let serialized = bincode::serialize(self).unwrap();
        let bytes = serialized.len();
        writer.write(&bytes.to_le_bytes())?;
        writer.write(&serialized)?;

        Ok(())
    }

    pub fn read_header<T: std::io::Read>(
        reader: &mut T,
    ) -> std::io::Result<BuiltObjectMetadata> {
        // let mut buffer = [0; 16];
        // reader.read(&mut buffer[0..4])?;
        // let count = u32::from_le_bytes(&buffer[0..4]);
        // let mut dependencies = Vec::with_capacity(count as usize);
        // for _ in 0..count {
        //     dependencies.push(ObjectId(reader.read_u128()?));
        // }
        //
        // let subresource_count = reader.read_u32()?;
        // let asset_type = Uuid::from_u128(reader.read_u128()?);

        let mut length_bytes = [0u8; 8];
        reader.read(&mut length_bytes)?;
        //let length = usize::from_le_bytes(length_bytes);

        let metadata = bincode::deserialize_from(reader).unwrap();
        Ok(metadata)

        // Ok(BuiltObjectMetadata {
        //     dependencies,
        //     subresource_count,
        //     asset_type
        // })
    }
}