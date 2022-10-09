// use crate::app::AppState;
// use crate::imgui_support::ImguiManager;
// use imgui::im_str;
// use imgui::sys::{
//     ImBitVector_Clear, ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking,
//     ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingOverMe, ImGuiWindowFlags_MenuBar,
//     ImGuiWindowFlags_NoCollapse, ImGuiWindowFlags_NoTitleBar, ImVec2,
// };
//
// use imgui::sys as is;
//
// fn to_c_id(id: imgui::Id) -> is::ImGuiID {
//     unsafe {
//         match id {
//             imgui::Id::Int(i) => is::igGetIDPtr(i as *const std::os::raw::c_void),
//             imgui::Id::Ptr(p) => is::igGetIDPtr(p),
//             imgui::Id::Str(s) => {
//                 let start = s.as_ptr() as *const std::os::raw::c_char;
//                 let end = start.add(s.len());
//                 is::igGetIDStrStr(start, end)
//             }
//         }
//     }
// }
//
// fn draw_menu_bar(
//     ui: &imgui::Ui,
//     _app_state: &mut AppState,
// ) {
//     ui.main_menu_bar(|| {
//         ui.menu(im_str!("File"), || {
//             imgui::MenuItem::new(im_str!("New")).build(ui);
//             imgui::MenuItem::new(im_str!("Open")).build(ui);
//             imgui::MenuItem::new(im_str!("Save")).build(ui);
//         });
//     });
// }
//
// fn draw_tool_bar(
//     ui: &imgui::Ui,
//     _app_state: &mut AppState,
// ) {
//     unsafe {
//         let viewport = is::igGetMainViewport();
//         let window_flags = is::ImGuiWindowFlags_NoScrollbar
//             | is::ImGuiWindowFlags_NoSavedSettings
//             | is::ImGuiWindowFlags_MenuBar;
//         let height = is::igGetFrameHeight();
//
//         if is::igBeginViewportSideBar(
//             im_str!("##ToolBar").as_ptr(),
//             viewport,
//             is::ImGuiDir_Up,
//             height,
//             window_flags as _,
//         ) {
//             ui.menu_bar(|| ui.text(im_str!("Tool Bar")));
//
//             is::igEnd();
//         }
//     }
// }
//
// fn draw_status_bar(
//     ui: &imgui::Ui,
//     _app_state: &mut AppState,
// ) {
//     unsafe {
//         let viewport = is::igGetMainViewport();
//         let window_flags = is::ImGuiWindowFlags_NoScrollbar
//             | is::ImGuiWindowFlags_NoSavedSettings
//             | is::ImGuiWindowFlags_MenuBar;
//         let height = is::igGetFrameHeight();
//
//         if is::igBeginViewportSideBar(
//             im_str!("##StatusBar").as_ptr(),
//             viewport,
//             is::ImGuiDir_Down,
//             height,
//             window_flags as _,
//         ) {
//             ui.menu_bar(|| ui.text(im_str!("Status Bar")));
//
//             is::igEnd();
//         }
//     }
// }
//
// fn setup_root_dockspace(
//     ui: &imgui::Ui,
//     _app_state: &mut AppState,
// ) {
//     unsafe {
//         let root_dockspace_id = is::igGetID_Str(im_str!("RootDockspace").as_ptr());
//         let main_viewport = imgui::sys::igGetMainViewport();
//         let work_pos = (*main_viewport).WorkPos.clone();
//         let work_size = (*main_viewport).WorkSize.clone();
//
//         imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowRounding as _, 0.0);
//         imgui::sys::igPushStyleVar_Float(imgui::sys::ImGuiStyleVar_WindowBorderSize as _, 0.0);
//         imgui::sys::igPushStyleVar_Vec2(
//             imgui::sys::ImGuiStyleVar_WindowPadding as _,
//             ImVec2::new(0.0, 0.0),
//         );
//
//         is::igSetNextWindowPos(work_pos, is::ImGuiCond_Always as _, ImVec2::zero());
//         is::igSetNextWindowSize(work_size, is::ImGuiCond_Always as _);
//
//         let draw_root_window = is::igBegin(
//             im_str!("Root Window").as_ptr(),
//             std::ptr::null_mut(),
//             (is::ImGuiWindowFlags_NoCollapse |
//                 //is::ImGuiWindowFlags_MenuBar |
//                 is::ImGuiWindowFlags_NoTitleBar |
//                 is::ImGuiWindowFlags_NoCollapse |
//                 is::ImGuiWindowFlags_NoResize |
//                 is::ImGuiWindowFlags_NoMove |
//                 is::ImGuiWindowFlags_NoDocking |
//                 is::ImGuiWindowFlags_NoBringToFrontOnFocus |
//                 is::ImGuiWindowFlags_NoNavFocus) as _,
//         );
//
//         // let root_window_token = imgui::Window::new(im_str!("Root Window"))
//         //     .position([work_pos.x, work_pos.y], imgui::Condition::Always)
//         //     .size([work_size.x, work_size.y], imgui::Condition::Always)
//         //     .flags(
//         //         imgui::WindowFlags::NO_TITLE_BAR |
//         //             imgui::WindowFlags::NO_COLLAPSE |
//         //             imgui::WindowFlags::NO_RESIZE |
//         //             imgui::WindowFlags::NO_MOVE |
//         //             imgui::WindowFlags::NO_DOCKING |
//         //             imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS |
//         //             imgui::WindowFlags::NO_NAV_FOCUS
//         //     )
//         //     // .draw_background(false)
//         //     // .resizable(false)
//         //     .begin(ui);
//
//         if draw_root_window {
//             if imgui::sys::igDockBuilderGetNode(root_dockspace_id).is_null() {
//                 imgui::sys::igDockBuilderRemoveNode(root_dockspace_id);
//                 imgui::sys::igDockBuilderAddNode(
//                     root_dockspace_id,
//                     imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace,
//                 );
//                 imgui::sys::igDockBuilderSetNodeSize(root_dockspace_id, work_size);
//
//                 let mut center_dockspace_id = root_dockspace_id;
//
//                 let mut right_dockspace_id = 0;
//                 is::igDockBuilderSplitNode(
//                     center_dockspace_id,
//                     is::ImGuiDir_Right,
//                     0.2,
//                     &mut right_dockspace_id,
//                     &mut center_dockspace_id,
//                 );
//
//                 let mut bottom_dockspace_id = 0;
//                 is::igDockBuilderSplitNode(
//                     center_dockspace_id,
//                     is::ImGuiDir_Down,
//                     0.4,
//                     &mut bottom_dockspace_id,
//                     &mut center_dockspace_id,
//                 );
//
//                 let mut left_dockspace_id = 0;
//                 is::igDockBuilderSplitNode(
//                     center_dockspace_id,
//                     is::ImGuiDir_Left,
//                     0.4,
//                     &mut left_dockspace_id,
//                     &mut center_dockspace_id,
//                 );
//
//                 is::igDockBuilderDockWindow(im_str!("RightWindow").as_ptr(), right_dockspace_id);
//                 is::igDockBuilderDockWindow(im_str!("AssetBrowser").as_ptr(), bottom_dockspace_id);
//                 //is::igDockBuilderDockWindow(im_str!("LogWindow").as_ptr(), bottom_dockspace_id);
//                 is::igDockBuilderDockWindow(im_str!("LeftWindow").as_ptr(), left_dockspace_id);
//                 is::igDockBuilderDockWindow(im_str!("NodeEditor").as_ptr(), center_dockspace_id);
//             }
//
//             is::igDockSpace(root_dockspace_id, ImVec2::zero(), 0, std::ptr::null());
//         }
//
//         is::igEnd();
//         is::igPopStyleVar(3);
//     }
// }
//
// fn setup_asset_browser_dockspace(
//     ui: &imgui::Ui,
//     _app_state: &mut AppState,
// ) {
//     unsafe {
//         let asset_browser_dockspace_id =
//             is::igGetID_Str(im_str!("AssetBrowserRootDockspace").as_ptr());
//         let main_viewport = imgui::sys::igGetMainViewport();
//         // let work_pos = (*main_viewport).WorkPos.clone();
//         // let work_size = (*main_viewport).WorkSize.clone();
//
//         imgui::sys::igPushStyleVar_Vec2(
//             imgui::sys::ImGuiStyleVar_WindowPadding as _,
//             ImVec2::new(0.0, 0.0),
//         );
//
//         // let root_window_token = imgui::Window::new(im_str!("AssetBrowser"))
//         //     .collapsible(false)
//         //     .menu_bar(true)
//         //     .begin(ui);
//
//         let draw_asset_browser = is::igBegin(
//             im_str!("AssetBrowser").as_ptr(),
//             std::ptr::null_mut(),
//             (is::ImGuiWindowFlags_NoCollapse | is::ImGuiWindowFlags_MenuBar) as _,
//         );
//
//         if draw_asset_browser {
//             let window = is::igGetCurrentWindow();
//             let window_size = (*window).Size;
//
//             if imgui::sys::igDockBuilderGetNode(asset_browser_dockspace_id).is_null() {
//                 imgui::sys::igDockBuilderRemoveNode(asset_browser_dockspace_id);
//                 imgui::sys::igDockBuilderAddNode(
//                     asset_browser_dockspace_id,
//                     imgui::sys::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_DockSpace,
//                 );
//                 imgui::sys::igDockBuilderSetNodeSize(asset_browser_dockspace_id, window_size);
//
//                 let mut left_dockspace_id = 0;
//                 let mut right_dockspace_id = 0;
//                 is::igDockBuilderSplitNode(
//                     asset_browser_dockspace_id,
//                     is::ImGuiDir_Right,
//                     0.2,
//                     &mut left_dockspace_id,
//                     &mut right_dockspace_id,
//                 );
//
//                 is::igDockBuilderDockWindow(
//                     im_str!("AssetBrowserLeft").as_ptr(),
//                     left_dockspace_id,
//                 );
//                 is::igDockBuilderDockWindow(
//                     im_str!("AssetBrowserRight").as_ptr(),
//                     right_dockspace_id,
//                 );
//
//                 let flags = is::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoTabBar
//                     | is::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDocking
//                     | is::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingOverMe
//                     | is::ImGuiDockNodeFlagsPrivate__ImGuiDockNodeFlags_NoDockingSplitMe;
//                 (*is::igDockBuilderGetNode(asset_browser_dockspace_id)).LocalFlags |= flags;
//                 (*is::igDockBuilderGetNode(left_dockspace_id)).LocalFlags |= flags;
//                 (*is::igDockBuilderGetNode(right_dockspace_id)).LocalFlags |= flags;
//             }
//
//             is::igDockSpace(
//                 asset_browser_dockspace_id,
//                 ImVec2::zero(),
//                 0,
//                 std::ptr::null(),
//             );
//         } else {
//             is::igDockSpace(
//                 asset_browser_dockspace_id,
//                 ImVec2::zero(),
//                 is::ImGuiDockNodeFlags__ImGuiDockNodeFlags_KeepAliveOnly as _,
//                 std::ptr::null(),
//             );
//
//             //is::igDockSpace(asset_browser_dockspace_id, ImVec2::zero(), is::ImGuiDockNodeFlags__ImGuiDockNodeFlags_KeepAliveOnly as _, std::ptr::null());
//             //is::igDockSpace(asset_browser_dockspace_id, ImVec2::zero(), is::ImGuiDockNodeFlags__ImGuiDockNodeFlags_KeepAliveOnly as _, std::ptr::null());
//         }
//
//         // println!("ptr {:?}", imgui::sys::igDockBuilderGetNode(asset_browser_dockspace_id));
//
//         // let ptr = imgui::sys::igDockBuilderGetNode(asset_browser_dockspace_id);
//         // let root_ptr = is::igDockNodeGetRootNode(ptr);
//         // let lfa = (*ptr).LastFrameAlive;
//         // let root_lfs = (*root_ptr).LastFrameAlive;
//         // println!("lfa {} root_lfa {}  framecount {}  ptr {:?}  root_ptr {:?}", lfa, root_lfs, is::igGetFrameCount(), ptr, root_ptr);
//
//         if draw_asset_browser {
//             ui.menu_bar(|| {
//                 ui.menu(im_str!("File"), || {
//                     imgui::MenuItem::new(im_str!("New")).build(ui);
//                     imgui::MenuItem::new(im_str!("Open")).build(ui);
//                     imgui::MenuItem::new(im_str!("Save")).build(ui);
//                 });
//             });
//         }
//
//         is::igEnd();
//
//         is::igPopStyleVar(1);
//     }
// }
//
// pub fn draw_imgui(
//     imgui_manager: &ImguiManager,
//     app_state: &mut AppState,
// ) {
//     //
//     //Draw an inspect window for the example struct
//     //
//     {
//         imgui_manager.with_ui(|ui: &mut imgui::Ui| {
//             draw_menu_bar(ui, app_state);
//             //draw_tool_bar(ui, app_state);
//             draw_status_bar(ui, app_state);
//
//             setup_root_dockspace(ui, app_state);
//             setup_asset_browser_dockspace(ui, app_state);
//
//             imgui::Window::new(im_str!("LeftWindow")).build(ui, || {
//                 ui.text("left");
//             });
//
//             imgui::Window::new(im_str!("RightWindow")).build(ui, || {
//                 ui.text("right");
//             });
//
//             imgui::Window::new(im_str!("LogWindow")).build(ui, || {});
//             imgui::Window::new(im_str!("AssetBrowserLeft")).build(ui, || {
//                 // unsafe {
//                 //     println!("dock node for AssetBrowserLeft {:?}", (*is::igGetCurrentWindow()).DockNode);
//                 // }
//
//                 // println!("ptr {:?}", imgui::sys::igDockBuilderGetNode(asset_browser_dockspace_id));
//                 // let lfa = (*imgui::sys::igDockBuilderGetNode(asset_browser_dockspace_id)).LastFrameAlive;
//                 // println!("lfa {}  framecount {}", lfa, is::igGetFrameCount());
//             });
//             imgui::Window::new(im_str!("AssetBrowserRight")).build(ui, || {});
//             imgui::Window::new(im_str!("NodeEditor"))
//                 .menu_bar(true)
//                 .build(ui, || {
//                     ui.menu_bar(|| {
//                         ui.menu(im_str!("File"), || {
//                             imgui::MenuItem::new(im_str!("New")).build(ui);
//                             imgui::MenuItem::new(im_str!("Open")).build(ui);
//                             imgui::MenuItem::new(im_str!("Save")).build(ui);
//                         });
//                     });
//                 });
//
//             unsafe {
//                 is::igShowMetricsWindow(std::ptr::null_mut());
//                 is::igShowStyleEditor(is::igGetStyle());
//             }
//         });
//     }
// }
