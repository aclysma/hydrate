// https://github.com/ocornut/imgui/issues/707 "yet another dark theme"
#[rustfmt::skip]
#[allow(dead_code)]
pub fn yet_another_dark_theme(style: &mut imgui::Style) {
    // https://github.com/ocornut/imgui/issues/707 "yet another dark theme"
    style.frame_rounding = 4.0;
    style.grab_rounding = 4.0;
    style[imgui::StyleColor::Text] = [0.95, 0.96, 0.98, 1.00];
    style[imgui::StyleColor::TextDisabled] = [0.36, 0.42, 0.47, 1.00];
    style[imgui::StyleColor::WindowBg] = [0.11, 0.15, 0.17, 1.00];
    style[imgui::StyleColor::ChildBg] = [0.15, 0.18, 0.22, 1.00];
    style[imgui::StyleColor::PopupBg] = [0.08, 0.08, 0.08, 0.94];
    style[imgui::StyleColor::Border] = [0.08, 0.10, 0.12, 1.00];
    style[imgui::StyleColor::BorderShadow] = [0.00, 0.00, 0.00, 0.00];
    style[imgui::StyleColor::FrameBg] = [0.20, 0.25, 0.29, 1.00];
    style[imgui::StyleColor::FrameBgHovered] = [0.12, 0.20, 0.28, 1.00];
    style[imgui::StyleColor::FrameBgActive] = [0.09, 0.12, 0.14, 1.00];
    style[imgui::StyleColor::TitleBg] = [0.09, 0.12, 0.14, 0.65];
    style[imgui::StyleColor::TitleBgActive] = [0.08, 0.10, 0.12, 1.00];
    style[imgui::StyleColor::TitleBgCollapsed] = [0.00, 0.00, 0.00, 0.51];
    style[imgui::StyleColor::MenuBarBg] = [0.15, 0.18, 0.22, 1.00];
    style[imgui::StyleColor::ScrollbarBg] = [0.02, 0.02, 0.02, 0.39];
    style[imgui::StyleColor::ScrollbarGrab] = [0.20, 0.25, 0.29, 1.00];
    style[imgui::StyleColor::ScrollbarGrabHovered] = [0.18, 0.22, 0.25, 1.00];
    style[imgui::StyleColor::ScrollbarGrabActive] = [0.09, 0.21, 0.31, 1.00];
    style[imgui::StyleColor::CheckMark] = [0.28, 0.56, 1.00, 1.00];
    style[imgui::StyleColor::SliderGrab] = [0.28, 0.56, 1.00, 1.00];
    style[imgui::StyleColor::SliderGrabActive] = [0.37, 0.61, 1.00, 1.00];
    style[imgui::StyleColor::Button] = [0.20, 0.25, 0.29, 1.00];
    style[imgui::StyleColor::ButtonHovered] = [0.28, 0.56, 1.00, 1.00];
    style[imgui::StyleColor::ButtonActive] = [0.06, 0.53, 0.98, 1.00];
    style[imgui::StyleColor::Header] = [0.20, 0.25, 0.29, 0.55];
    style[imgui::StyleColor::HeaderHovered] = [0.26, 0.59, 0.98, 0.80];
    style[imgui::StyleColor::HeaderActive] = [0.26, 0.59, 0.98, 1.00];
    style[imgui::StyleColor::Separator] = [0.20, 0.25, 0.29, 1.00];
    style[imgui::StyleColor::SeparatorHovered] = [0.10, 0.40, 0.75, 0.78];
    style[imgui::StyleColor::SeparatorActive] = [0.10, 0.40, 0.75, 1.00];
    style[imgui::StyleColor::ResizeGrip] = [0.26, 0.59, 0.98, 0.25];
    style[imgui::StyleColor::ResizeGripHovered] = [0.26, 0.59, 0.98, 0.67];
    style[imgui::StyleColor::ResizeGripActive] = [0.26, 0.59, 0.98, 0.95];
    style[imgui::StyleColor::Tab] = [0.11, 0.15, 0.17, 1.00];
    style[imgui::StyleColor::TabHovered] = [0.26, 0.59, 0.98, 0.80];
    style[imgui::StyleColor::TabActive] = [0.20, 0.25, 0.29, 1.00];
    style[imgui::StyleColor::TabUnfocused] = [0.11, 0.15, 0.17, 1.00];
    style[imgui::StyleColor::TabUnfocusedActive] = [0.11, 0.15, 0.17, 1.00];
    style[imgui::StyleColor::PlotLines] = [0.61, 0.61, 0.61, 1.00];
    style[imgui::StyleColor::PlotLinesHovered] = [1.00, 0.43, 0.35, 1.00];
    style[imgui::StyleColor::PlotHistogram] = [0.90, 0.70, 0.00, 1.00];
    style[imgui::StyleColor::PlotHistogramHovered] = [1.00, 0.60, 0.00, 1.00];
    style[imgui::StyleColor::TextSelectedBg] = [0.26, 0.59, 0.98, 0.35];
    style[imgui::StyleColor::DragDropTarget] = [1.00, 1.00, 0.00, 0.90];
    style[imgui::StyleColor::NavHighlight] = [0.26, 0.59, 0.98, 1.00];
    style[imgui::StyleColor::NavWindowingHighlight] = [1.00, 1.00, 1.00, 0.70];
    style[imgui::StyleColor::NavWindowingDimBg] = [0.80, 0.80, 0.80, 0.20];
    style[imgui::StyleColor::ModalWindowDimBg] = [0.80, 0.80, 0.80, 0.35];
}

#[rustfmt::skip]
pub fn vsdark_theme(style: &mut imgui::Style) {
    // https://github.com/ocornut/imgui/issues/707 "yet another dark theme"
    style.window_rounding = 0.0;
    style.child_rounding = 0.0;
    style.frame_rounding = 0.0;
    style.grab_rounding = 0.0;
    style.popup_rounding = 0.0;
    style.scrollbar_rounding = 0.0;
    style.tab_rounding = 0.0;

    fn rgb_to_4xf32(r: f32, g: f32, b: f32) -> [f32; 4] {
        [r / 255.0, g / 255.0, b / 255.0, 1.0]
    }

    let bg_color = rgb_to_4xf32(37.0, 37.0, 38.0);
    let light_bg_color = rgb_to_4xf32(82.0, 82.0, 85.0);
    let very_light_bg_color = rgb_to_4xf32(90.0, 90.0, 95.0);

    let panel_color = rgb_to_4xf32(51.0, 51.0, 55.0);
    let panel_hover_color = rgb_to_4xf32(29.0, 151.0, 236.0);
    let panel_active_color = rgb_to_4xf32(0.0, 119.0, 200.0);

    let text_color = rgb_to_4xf32(255.0, 255.0, 255.0);
    let text_disabled_color = rgb_to_4xf32(151.0, 151.0, 151.0);
    let border_color = rgb_to_4xf32(78.0, 78.0, 78.0);

    // Add some color so that selected list item/tree node elements in are clearly distinct. This
    // also affects the collapsable headers
    //let header = rgb_to_4xf32(70.0, 70.0, 110.0);
    let header = rgb_to_4xf32(46.0, 78.0, 117.0);
    // Make buttons pop a bit more (and match the scrollbar grab color)
    let button = light_bg_color; //rgb_to_4xf32(70.0, 70.0, 80.0);

    style[imgui::StyleColor::Text]                 = text_color;
    style[imgui::StyleColor::TextDisabled]         = text_disabled_color;
    style[imgui::StyleColor::TextSelectedBg]       = panel_active_color;
    style[imgui::StyleColor::WindowBg]             = bg_color;
    style[imgui::StyleColor::ChildBg]              = bg_color;
    style[imgui::StyleColor::PopupBg]              = bg_color;
    style[imgui::StyleColor::Border]               = border_color;
    style[imgui::StyleColor::BorderShadow]         = border_color;
    style[imgui::StyleColor::FrameBg]              = panel_color;
    style[imgui::StyleColor::FrameBgHovered]       = panel_hover_color;
    style[imgui::StyleColor::FrameBgActive]        = panel_active_color;
    style[imgui::StyleColor::TitleBg]              = bg_color;
    style[imgui::StyleColor::TitleBgActive]        = bg_color;
    style[imgui::StyleColor::TitleBgCollapsed]     = bg_color;
    style[imgui::StyleColor::MenuBarBg]            = panel_color;
    style[imgui::StyleColor::ScrollbarBg]          = panel_color;
    style[imgui::StyleColor::ScrollbarGrab]        = light_bg_color;
    style[imgui::StyleColor::ScrollbarGrabHovered] = very_light_bg_color;
    style[imgui::StyleColor::ScrollbarGrabActive]  = very_light_bg_color;
    style[imgui::StyleColor::CheckMark]            = panel_active_color;
    style[imgui::StyleColor::SliderGrab]           = panel_hover_color;
    style[imgui::StyleColor::SliderGrabActive]     = panel_active_color;
    style[imgui::StyleColor::Button]               = button;
    style[imgui::StyleColor::ButtonHovered]        = panel_hover_color;
    style[imgui::StyleColor::ButtonActive]         = panel_hover_color;
    style[imgui::StyleColor::Header]               = header;
    style[imgui::StyleColor::HeaderHovered]        = panel_hover_color;
    style[imgui::StyleColor::HeaderActive]         = panel_active_color;
    style[imgui::StyleColor::Separator]            = border_color;
    style[imgui::StyleColor::SeparatorHovered]     = border_color;
    style[imgui::StyleColor::SeparatorActive]      = border_color;
    style[imgui::StyleColor::ResizeGrip]           = bg_color;
    style[imgui::StyleColor::ResizeGripHovered]    = panel_color;
    style[imgui::StyleColor::ResizeGripActive]     = light_bg_color;
    style[imgui::StyleColor::PlotLines]            = panel_active_color;
    style[imgui::StyleColor::PlotLinesHovered]     = panel_hover_color;
    style[imgui::StyleColor::PlotHistogram]        = panel_active_color;
    style[imgui::StyleColor::PlotHistogramHovered] = panel_hover_color;
    //style[imgui::StyleColor::ModalWindowDarkening] = bg_color;

    style[imgui::StyleColor::DragDropTarget]       = bg_color;
    style[imgui::StyleColor::NavHighlight]         = bg_color;
    style[imgui::StyleColor::DockingPreview]       = panel_active_color;
    style[imgui::StyleColor::Tab]                  = bg_color;
    style[imgui::StyleColor::TabActive]            = panel_active_color;
    style[imgui::StyleColor::TabUnfocused]         = bg_color;
    style[imgui::StyleColor::TabUnfocusedActive]   = panel_active_color;
    style[imgui::StyleColor::TabHovered]           = panel_hover_color;


    // dim the background out with a darker but semi-neutral color
    style[imgui::StyleColor::ModalWindowDimBg] = [0.2, 0.2, 0.2, 0.7];
    // gray the background out with a brighter color
    //style[imgui::StyleColor::ModalWindowDimBg] = [0.8, 0.8, 0.8, 0.35];
}

// https://github.com/ocornut/imgui/issues/707 "charcoal"
#[rustfmt::skip]
#[allow(dead_code)]
pub fn charcoal_theme(style: &mut imgui::Style) {
    style[imgui::StyleColor::Text]                   = [1.000, 1.000, 1.000, 1.000];
    style[imgui::StyleColor::TextDisabled]           = [0.500, 0.500, 0.500, 1.000];
    style[imgui::StyleColor::WindowBg]               = [0.180, 0.180, 0.180, 1.000];
    style[imgui::StyleColor::ChildBg]                = [0.280, 0.280, 0.280, 0.000];
    style[imgui::StyleColor::PopupBg]                = [0.313, 0.313, 0.313, 1.000];
    style[imgui::StyleColor::Border]                 = [0.266, 0.266, 0.266, 1.000];
    style[imgui::StyleColor::BorderShadow]           = [0.000, 0.000, 0.000, 0.000];
    style[imgui::StyleColor::FrameBg]                = [0.160, 0.160, 0.160, 1.000];
    style[imgui::StyleColor::FrameBgHovered]         = [0.200, 0.200, 0.200, 1.000];
    style[imgui::StyleColor::FrameBgActive]          = [0.280, 0.280, 0.280, 1.000];
    style[imgui::StyleColor::TitleBg]                = [0.148, 0.148, 0.148, 1.000];
    style[imgui::StyleColor::TitleBgActive]          = [0.148, 0.148, 0.148, 1.000];
    style[imgui::StyleColor::TitleBgCollapsed]       = [0.148, 0.148, 0.148, 1.000];
    style[imgui::StyleColor::MenuBarBg]              = [0.195, 0.195, 0.195, 1.000];
    style[imgui::StyleColor::ScrollbarBg]            = [0.160, 0.160, 0.160, 1.000];
    style[imgui::StyleColor::ScrollbarGrab]          = [0.277, 0.277, 0.277, 1.000];
    style[imgui::StyleColor::ScrollbarGrabHovered]   = [0.300, 0.300, 0.300, 1.000];
    style[imgui::StyleColor::ScrollbarGrabActive]    = [1.000, 0.391, 0.000, 1.000];
    style[imgui::StyleColor::CheckMark]              = [1.000, 1.000, 1.000, 1.000];
    style[imgui::StyleColor::SliderGrab]             = [0.391, 0.391, 0.391, 1.000];
    style[imgui::StyleColor::SliderGrabActive]       = [1.000, 0.391, 0.000, 1.000];
    style[imgui::StyleColor::Button]                 = [1.000, 1.000, 1.000, 0.000];
    style[imgui::StyleColor::ButtonHovered]          = [1.000, 1.000, 1.000, 0.156];
    style[imgui::StyleColor::ButtonActive]           = [1.000, 1.000, 1.000, 0.391];
    style[imgui::StyleColor::Header]                 = [0.313, 0.313, 0.313, 1.000];
    style[imgui::StyleColor::HeaderHovered]          = [0.469, 0.469, 0.469, 1.000];
    style[imgui::StyleColor::HeaderActive]           = [0.469, 0.469, 0.469, 1.000];
    style[imgui::StyleColor::Separator]              = style[imgui::StyleColor::Border];
    style[imgui::StyleColor::SeparatorHovered]       = [0.391, 0.391, 0.391, 1.000];
    style[imgui::StyleColor::SeparatorActive]        = [1.000, 0.391, 0.000, 1.000];
    style[imgui::StyleColor::ResizeGrip]             = [1.000, 1.000, 1.000, 0.250];
    style[imgui::StyleColor::ResizeGripHovered]      = [1.000, 1.000, 1.000, 0.670];
    style[imgui::StyleColor::ResizeGripActive]       = [1.000, 0.391, 0.000, 1.000];
    style[imgui::StyleColor::Tab]                    = [0.098, 0.098, 0.098, 1.000];
    style[imgui::StyleColor::TabHovered]             = [0.352, 0.352, 0.352, 1.000];
    style[imgui::StyleColor::TabActive]              = [0.195, 0.195, 0.195, 1.000];
    style[imgui::StyleColor::TabUnfocused]           = [0.098, 0.098, 0.098, 1.000];
    style[imgui::StyleColor::TabUnfocusedActive]     = [0.195, 0.195, 0.195, 1.000];
    //style[imgui::StyleColor::DockingPreview]         = [1.000, 0.391, 0.000, 0.781];
    //style[imgui::StyleColor::DockingEmptyBg]         = [0.180, 0.180, 0.180, 1.000];
    style[imgui::StyleColor::PlotLines]              = [0.469, 0.469, 0.469, 1.000];
    style[imgui::StyleColor::PlotLinesHovered]       = [1.000, 0.391, 0.000, 1.000];
    style[imgui::StyleColor::PlotHistogram]          = [0.586, 0.586, 0.586, 1.000];
    style[imgui::StyleColor::PlotHistogramHovered]   = [1.000, 0.391, 0.000, 1.000];
    style[imgui::StyleColor::TextSelectedBg]         = [1.000, 1.000, 1.000, 0.156];
    style[imgui::StyleColor::DragDropTarget]         = [1.000, 0.391, 0.000, 1.000];
    style[imgui::StyleColor::NavHighlight]           = [1.000, 0.391, 0.000, 1.000];
    style[imgui::StyleColor::NavWindowingHighlight]  = [1.000, 0.391, 0.000, 1.000];
    style[imgui::StyleColor::NavWindowingDimBg]      = [0.000, 0.000, 0.000, 0.586];
    style[imgui::StyleColor::ModalWindowDimBg]       = [0.000, 0.000, 0.000, 0.586];

    style.child_rounding = 4.0;
    style.frame_border_size = 1.0;
    style.frame_rounding = 2.0;
    style.grab_min_size = 7.0;
    style.popup_rounding = 2.0;
    style.scrollbar_rounding = 12.0;
    style.scrollbar_size = 13.0;
    style.tab_border_size = 1.0;
    style.tab_rounding = 0.0;
    style.window_rounding = 4.0;
}

// https://github.com/ocornut/imgui/issues/707 "corporate_gray"
#[rustfmt::skip]
#[allow(dead_code)]
pub fn corporate_gray_theme(style: &mut imgui::Style) {
    // 0 = FLAT APPEARENCE
    // 1 = MORE "3D" LOOK
    let is_3d = 0.0;

    style[imgui::StyleColor::Text]                   = [1.00, 1.00, 1.00, 1.00];
    style[imgui::StyleColor::TextDisabled]           = [0.40, 0.40, 0.40, 1.00];
    style[imgui::StyleColor::ChildBg]                = [0.25, 0.25, 0.25, 1.00];
    style[imgui::StyleColor::WindowBg]               = [0.25, 0.25, 0.25, 1.00];
    style[imgui::StyleColor::PopupBg]                = [0.25, 0.25, 0.25, 1.00];
    style[imgui::StyleColor::Border]                 = [0.12, 0.12, 0.12, 0.71];
    style[imgui::StyleColor::BorderShadow]           = [1.00, 1.00, 1.00, 0.06];
    style[imgui::StyleColor::FrameBg]                = [0.42, 0.42, 0.42, 0.54];
    style[imgui::StyleColor::FrameBgHovered]         = [0.42, 0.42, 0.42, 0.40];
    style[imgui::StyleColor::FrameBgActive]          = [0.56, 0.56, 0.56, 0.67];
    style[imgui::StyleColor::TitleBg]                = [0.19, 0.19, 0.19, 1.00];
    style[imgui::StyleColor::TitleBgActive]          = [0.22, 0.22, 0.22, 1.00];
    style[imgui::StyleColor::TitleBgCollapsed]       = [0.17, 0.17, 0.17, 0.90];
    style[imgui::StyleColor::MenuBarBg]              = [0.335, 0.335, 0.335, 1.000];
    style[imgui::StyleColor::ScrollbarBg]            = [0.24, 0.24, 0.24, 0.53];
    style[imgui::StyleColor::ScrollbarGrab]          = [0.41, 0.41, 0.41, 1.00];
    style[imgui::StyleColor::ScrollbarGrabHovered]   = [0.52, 0.52, 0.52, 1.00];
    style[imgui::StyleColor::ScrollbarGrabActive]    = [0.76, 0.76, 0.76, 1.00];
    style[imgui::StyleColor::CheckMark]              = [0.65, 0.65, 0.65, 1.00];
    style[imgui::StyleColor::SliderGrab]             = [0.52, 0.52, 0.52, 1.00];
    style[imgui::StyleColor::SliderGrabActive]       = [0.64, 0.64, 0.64, 1.00];
    style[imgui::StyleColor::Button]                 = [0.54, 0.54, 0.54, 0.35];
    style[imgui::StyleColor::ButtonHovered]          = [0.52, 0.52, 0.52, 0.59];
    style[imgui::StyleColor::ButtonActive]           = [0.76, 0.76, 0.76, 1.00];
    style[imgui::StyleColor::Header]                 = [0.38, 0.38, 0.38, 1.00];
    style[imgui::StyleColor::HeaderHovered]          = [0.47, 0.47, 0.47, 1.00];
    style[imgui::StyleColor::HeaderActive]           = [0.76, 0.76, 0.76, 0.77];
    style[imgui::StyleColor::Separator]              = [0.000, 0.000, 0.000, 0.137];
    style[imgui::StyleColor::SeparatorHovered]       = [0.700, 0.671, 0.600, 0.290];
    style[imgui::StyleColor::SeparatorActive]        = [0.702, 0.671, 0.600, 0.674];
    style[imgui::StyleColor::ResizeGrip]             = [0.26, 0.59, 0.98, 0.25];
    style[imgui::StyleColor::ResizeGripHovered]      = [0.26, 0.59, 0.98, 0.67];
    style[imgui::StyleColor::ResizeGripActive]       = [0.26, 0.59, 0.98, 0.95];
    style[imgui::StyleColor::PlotLines]              = [0.61, 0.61, 0.61, 1.00];
    style[imgui::StyleColor::PlotLinesHovered]       = [1.00, 0.43, 0.35, 1.00];
    style[imgui::StyleColor::PlotHistogram]          = [0.90, 0.70, 0.00, 1.00];
    style[imgui::StyleColor::PlotHistogramHovered]   = [1.00, 0.60, 0.00, 1.00];
    style[imgui::StyleColor::TextSelectedBg]         = [0.73, 0.73, 0.73, 0.35];
    style[imgui::StyleColor::ModalWindowDimBg]       = [0.80, 0.80, 0.80, 0.35];
    style[imgui::StyleColor::DragDropTarget]         = [1.00, 1.00, 0.00, 0.90];
    style[imgui::StyleColor::NavHighlight]           = [0.26, 0.59, 0.98, 1.00];
    style[imgui::StyleColor::NavWindowingHighlight]  = [1.00, 1.00, 1.00, 0.70];
    style[imgui::StyleColor::NavWindowingDimBg]      = [0.80, 0.80, 0.80, 0.20];

    style.popup_rounding = 3.0;

    style.window_padding = [4.0, 4.0];
    style.frame_padding  = [6.0, 4.0];
    style.item_spacing   = [6.0, 2.0];

    style.scrollbar_size = 18.0;

    style.window_border_size = 1.0;
    style.child_border_size  = 1.0;
    style.popup_border_size  = 1.0;
    style.frame_border_size  = is_3d;

    style.window_rounding    = 3.0;
    style.child_rounding     = 3.0;
    style.frame_rounding     = 3.0;
    style.scrollbar_rounding = 2.0;
    style.grab_rounding      = 3.0;
    /*
        #ifdef IMGUI_HAS_DOCK
        style.TabBorderSize = is3D;
        style.TabRounding   = 3;
        colors[ImGuiCol_DockingEmptyBg]     = ImVec4(0.38f, 0.38f, 0.38f, 1.00f);
        colors[ImGuiCol_Tab]                = ImVec4(0.25f, 0.25f, 0.25f, 1.00f);
        colors[ImGuiCol_TabHovered]         = ImVec4(0.40f, 0.40f, 0.40f, 1.00f);
        colors[ImGuiCol_TabActive]          = ImVec4(0.33f, 0.33f, 0.33f, 1.00f);
        colors[ImGuiCol_TabUnfocused]       = ImVec4(0.25f, 0.25f, 0.25f, 1.00f);
        colors[ImGuiCol_TabUnfocusedActive] = ImVec4(0.33f, 0.33f, 0.33f, 1.00f);
        colors[ImGuiCol_DockingPreview]     = ImVec4(0.85f, 0.85f, 0.85f, 0.28f);
        if (ImGui::GetIO().ConfigFlags & ImGuiConfigFlags_ViewportsEnable)
        {
            style.WindowRounding = 0.0f;
            style.Colors[ImGuiCol_WindowBg].w = 1.0f;
        }
        #endif
    */
}

#[rustfmt::skip]
#[allow(dead_code)]
pub fn custom_theme(style: &mut imgui::Style) {
    yet_another_dark_theme(style);

    fn unconvert_imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
        let x = col[0].powf(1.0/2.2);
        let y = col[1].powf(1.0/2.2);
        let z = col[2].powf(1.0/2.2);
        //let w = 1.0 - (1.0 - col[3]).powf(1.0/2.2);
        let w = 1.0 - (1.0 - col[3]).powf(1.0/2.2);
        [x, y, z, w]
    }

    let header = unconvert_imgui_gamma_to_linear([0.106, 0.188, 0.270, 0.827]);
    let resize_grip = unconvert_imgui_gamma_to_linear([0.0, 0.0, 0.0, 0.0]);

    style.frame_rounding = 4.0;
    style.grab_rounding = 4.0;
    style.grab_min_size = 8.0;

    style[imgui::StyleColor::FrameBgActive] = style[imgui::StyleColor::FrameBgHovered];

    style[imgui::StyleColor::ButtonHovered] = style[imgui::StyleColor::SeparatorHovered];
    style[imgui::StyleColor::ButtonActive] = style[imgui::StyleColor::SeparatorHovered];
    style[imgui::StyleColor::HeaderHovered] = style[imgui::StyleColor::SeparatorHovered];
    style[imgui::StyleColor::HeaderActive] = style[imgui::StyleColor::SeparatorHovered];
    style[imgui::StyleColor::TabHovered] = style[imgui::StyleColor::SeparatorHovered];
    style[imgui::StyleColor::Header] = header;
    style[imgui::StyleColor::ResizeGrip] = resize_grip;

    style[imgui::StyleColor::MenuBarBg] = unconvert_imgui_gamma_to_linear([8.0/255.0, 15.0/255.0, 28.0/255.0, 1.0]);
    style[imgui::StyleColor::TabActive] = unconvert_imgui_gamma_to_linear([9.0/255.0, 20.0/255.0, 27.0/255.0, 1.0]);
    style[imgui::StyleColor::TabUnfocusedActive] = unconvert_imgui_gamma_to_linear([9.0/255.0, 20.0/255.0, 27.0/255.0, 1.0]);
    style[imgui::StyleColor::TitleBg] = unconvert_imgui_gamma_to_linear([1.0/255.0, 2.0/255.0, 3.0/255.0, 1.0]);
}
