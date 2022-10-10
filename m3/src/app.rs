use crate::data_source::{FileSystemPackage};
use crate::test_data::TestData;

// This struct is a simple example of something that can be inspected
pub struct AppState {
    // pub db: ObjectDb,
    // pub prototype_obj: ObjectId,
    // pub instance_obj: ObjectId,

    pub redock_windows: bool,
    pub file_system_package: FileSystemPackage,
    pub test_data_nexdb: TestData,
    pub show_imgui_demo_window: bool,
    //TODO: New DB type here, update draw_2_pane_view to call draw_inspector with new data
}

impl AppState {
    pub fn new(file_system_package: FileSystemPackage, test_data: TestData) -> Self {
        AppState {
            redock_windows: true,
            file_system_package,
            test_data_nexdb: test_data,
            show_imgui_demo_window: false
        }
    }
}