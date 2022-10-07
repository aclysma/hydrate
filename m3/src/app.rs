// This struct is a simple example of something that can be inspected
pub struct AppState {
    // pub db: ObjectDb,
    // pub prototype_obj: ObjectId,
    // pub instance_obj: ObjectId,
    pub test_data_nexdb: crate::test_data_nexdb::TestData,
    //TODO: New DB type here, update draw_2_pane_view to call draw_inspector with new data
}
