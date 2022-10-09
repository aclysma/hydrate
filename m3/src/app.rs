use crate::test_data::TestData;

// This struct is a simple example of something that can be inspected
pub struct AppState {
    // pub db: ObjectDb,
    // pub prototype_obj: ObjectId,
    // pub instance_obj: ObjectId,

    pub redock_windows: bool,

    pub test_data_nexdb: TestData,
    //TODO: New DB type here, update draw_2_pane_view to call draw_inspector with new data
}

impl AppState {
    pub fn new(test_data: TestData) -> Self {
        AppState {
            redock_windows: true,
            test_data_nexdb: test_data
        }
    }
}