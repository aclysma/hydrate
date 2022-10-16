pub mod imgui_manager;
pub use imgui_manager::*;

pub mod themes;

struct ImguiContext {
    pub imgui_context: imgui::Context,
    pub imnodes_context: imnodes::Context,
}

fn init_imgui(window: &winit::window::Window) -> ImguiContext {
    use imgui::Context;

    let mut imgui_context = Context::create();
    let imnodes_context = imnodes::Context::new();

    {
        // Fix incorrect colors with sRGB framebuffer
        fn imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
            let x = col[0].powf(2.2);
            let y = col[1].powf(2.2);
            let z = col[2].powf(2.2);
            let w = col[3]; //1.0 - (1.0 - col[3]).powf(2.2);
            [x, y, z, w]
        }

        let style = imgui_context.style_mut();
        // style.use_dark_colors();
        // for col in 0..style.colors.len() {
        //     style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
        // }

        crate::imgui_support::themes::vsdark_theme(style);
        //crate::imgui_support::themes::custom_theme(style);
        for col in 0..style.colors.len() {
            style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
        }
    }

    imgui_context.set_ini_filename(None);

    // In the examples we only use integer DPI factors, because the UI can get very blurry
    // otherwise. This might or might not be what you want in a real application.
    let scale_factor = window.scale_factor().round();
    let font_size = (16.0 * scale_factor) as f32;

    let feather_ttf = {
        // https://pixinvent.com/apex-angular-4-bootstrap-admin-template/html-demo-4/icons-feather.html
        let mut font_config = imgui::FontConfig::default();
        const ICON_GLYPH_RANGE_FEATHER: [u32; 3] = [0xe81b, 0xe92a, 0];
        font_config.glyph_ranges = imgui::FontGlyphRanges::from_slice(&ICON_GLYPH_RANGE_FEATHER);
        imgui::FontSource::TtfData {
            data: include_bytes!("../../../fonts/feather.ttf"),
            size_pixels: font_size,
            config: Some(font_config),
        }
        //imgui.fonts().add_font(&[]);
    };

    let material_ttf = {
        // Material icons
        const ICON_GLYPH_RANGE_MATERIAL: [u32; 15] = [
            //0xfd24, 0xfd34, // transform/rotate icons
            0xe2c7, 0xe2c7, // folder
            0xf3e4, 0xf3e4, // pause
            0xf40a, 0xf40a, // play
            0xf1b5, 0xf1b5, // select
            0xfd25, 0xfd25, // translate
            0xfd74, 0xfd74, // rotate
            0xfa67, 0xfa67, // scale
            0,
        ];
        let mut font_config = imgui::FontConfig::default();
        font_config.glyph_ranges = imgui::FontGlyphRanges::from_slice(&ICON_GLYPH_RANGE_MATERIAL);
        font_config.glyph_offset = [0.0, 6.0];
        font_config.glyph_min_advance_x = 16.0;

        imgui::FontSource::TtfData {
            data: include_bytes!("../../../fonts/materialdesignicons-webfont.ttf"),
            size_pixels: font_size,
            config: Some(font_config),
        }
    };

    let mplus_ttf = imgui::FontSource::TtfData {
        data: include_bytes!("../../../fonts/mplus-1p-regular.ttf"),
        size_pixels: font_size,
        config: None,
    };

    imgui_context
        .fonts()
        .add_font(&[mplus_ttf, feather_ttf, material_ttf]);

    imgui_context.io_mut().font_global_scale = (1.0 / scale_factor) as f32;

    ImguiContext {
        imgui_context,
        imnodes_context,
    }
}

pub fn init_imgui_manager(window: &winit::window::Window) -> ImguiManager {
    let mut imgui_context = init_imgui(&window);
    let mut imgui_platform =
        imgui_winit_support::WinitPlatform::init(&mut imgui_context.imgui_context);

    imgui_platform.attach_window(
        imgui_context.imgui_context.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Rounded,
    );

    imgui_context.imgui_context.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;
    imgui_context.imgui_context.io_mut().backend_flags |=
        imgui::BackendFlags::RENDERER_HAS_VTX_OFFSET;

    ImguiManager::new(
        imgui_context.imgui_context,
        imgui_context.imnodes_context,
        imgui_platform,
    )
}
