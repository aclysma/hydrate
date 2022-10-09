use crate::test_data::TestData;

// This struct is a simple example of something that can be inspected
pub struct AppState {
    // pub db: ObjectDb,
    // pub prototype_obj: ObjectId,
    // pub instance_obj: ObjectId,

    pub redock_windows: bool,
    pub splitter_size1: f32,
    pub splitter_size2: f32,

    pub test_data_nexdb: TestData,
    //TODO: New DB type here, update draw_2_pane_view to call draw_inspector with new data
}

impl AppState {
    pub fn new(test_data: TestData) -> Self {
        AppState {
            redock_windows: true,
            splitter_size1: 100.0,
            splitter_size2: 100.0,
            test_data_nexdb: test_data
        }
    }
}