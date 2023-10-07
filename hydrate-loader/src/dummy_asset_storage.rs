// use hydrate_model::{HashMap, ObjectId};
//
// #[derive(Default)]
// pub struct DummyAssetStorage {
//     //preparing: HashMap<ObjectId, Vec<u8>>,
//     //committed: HashMap<ObjectId, Vec<u8>>,
// }
//
// impl DummyAssetStorage {
//     pub fn prepare(
//         &mut self,
//         object_id: ObjectId,
//         data: Vec<u8>,
//     ) {
//         log::info!("prepare object {:?}", object_id);
//     }
//
//     pub fn commmit(
//         &mut self,
//         object_id: ObjectId,
//     ) {
//         log::info!("commit object {:?}", object_id);
//     }
//
//     pub fn free(
//         &mut self,
//         object_id: ObjectId,
//     ) {
//         log::info!("free object {:?}", object_id);
//     }
// }
