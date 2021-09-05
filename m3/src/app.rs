use refdb::*;

// This struct is a simple example of something that can be inspected
pub struct AppState {
    pub db: ObjectDb,
    pub prototype_obj: ObjectId,
    pub instance_obj: ObjectId,
}