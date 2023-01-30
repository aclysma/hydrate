use imgui::sys as imgui_sys;

use std::sync::Arc;
use std::sync::Mutex;

// Inner state for ImguiManager, which will be protected by a Mutex. Mutex protection required since
// this object is Send but not Sync
struct Inner {
    context: imgui::Context,
    imnodes_context: imnodes::Context,

    // Pointer to the font atlas. Assuming no direct calls to C imgui interface, this pointer is
    // valid as long as context is not destroyed
    font_atlas_texture: *mut imgui::FontAtlasTexture<'static>,

    // Pointer to the current UI. Assuming no direct calls to C imgui interface, this pointer is
    // valid as long as context is not destroyed, and a frame has started and not ended
    ui: Option<*mut imgui::Ui<'static>>,

    // Handles the integration between imgui and winit
    platform: imgui_winit_support::WinitPlatform,

    // These are all refreshed when frame is started
    want_capture_keyboard: bool,
    want_capture_mouse: bool,
    want_set_mouse_pos: bool,
    want_text_input: bool,
}

// Rust assumes pointers in Inner are not safe to send, so we need to explicitly impl that here
unsafe impl Send for Inner {}

impl Drop for Inner {
    fn drop(&mut self) {
        let mut ui = None;
        std::mem::swap(&mut self.ui, &mut ui);

        // Drop the UI call if it exists
        if let Some(ui) = ui {
            let _ui = unsafe { Box::from_raw(ui) };
        }

        // Drop the font atlas
        unsafe { Box::from_raw(self.font_atlas_texture) };
    }
}

//TODO: Investigate usage of channels/draw lists
#[derive(Clone)]
pub struct ImguiManager {
    inner: Arc<Mutex<Inner>>,
}

// Wraps imgui (and winit integration logic)
impl ImguiManager {
    // imgui and winit platform are expected to be pre-configured
    pub fn new(
        mut imgui_context: imgui::Context,
        imnodes_context: imnodes::Context,
        platform: imgui_winit_support::WinitPlatform,
    ) -> Self {
        // Ensure font atlas is built and cache a pointer to it
        let font_atlas_texture = {
            let mut fonts = imgui_context.fonts();
            let font_atlas_texture = Box::new(fonts.build_rgba32_texture());
            log::info!("Building ImGui font atlas");

            // Remove the lifetime of the texture. (We're assuming we have ownership over it
            // now since imgui_context is being passed to us)
            let font_atlas_texture: *mut imgui::FontAtlasTexture =
                Box::into_raw(font_atlas_texture);
            #[allow(clippy::transmute_ptr_to_ptr)]
            let font_atlas_texture: *mut imgui::FontAtlasTexture<'static> =
                unsafe { std::mem::transmute(font_atlas_texture) };
            font_atlas_texture
        };

        ImguiManager {
            inner: Arc::new(Mutex::new(Inner {
                context: imgui_context,
                imnodes_context,
                font_atlas_texture,
                ui: None,
                platform,
                want_capture_keyboard: false,
                want_capture_mouse: false,
                want_set_mouse_pos: false,
                want_text_input: false,
            })),
        }
    }

    // Call when a winit event is received
    //TODO: Taking a lock per event sucks
    pub fn handle_event<T>(
        &self,
        window: &winit::window::Window,
        event: &winit::event::Event<T>,
    ) {
        let mut inner = self.inner.lock().unwrap();
        let inner = &mut *inner;
        let context = &mut inner.context;
        let platform = &mut inner.platform;

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::ReceivedCharacter(ch),
                ..
            } if *ch == '\u{7f}' => {
                // Do not pass along a backspace character
                // This hack can be removed when https://github.com/Gekkio/imgui-rs/pull/253 is
                // implemented upstream and I switch to using it
            }
            _ => {
                platform.handle_event(context.io_mut(), &window, &event);
            }
        }
    }

    // Allows access to the context without caller needing to be aware of locking
    #[allow(dead_code)]
    pub fn with_context<F>(
        &self,
        f: F,
    ) where
        F: FnOnce(&mut imgui::Context),
    {
        let mut inner = self.inner.lock().unwrap();
        (f)(&mut inner.context);
    }

    pub fn new_imnodes_editor(&self) -> imnodes::EditorContext {
        let inner = self.inner.lock().unwrap();
        let editor = inner.imnodes_context.create_editor();

        editor.set_style_colors_dark();
        //editor.set_style_colors_classic();
        //editor.set_style_colors_light();

        // Fix gamma on all the colors
        for color in &mut editor.get_style().colors {
            // Fix incorrect colors with sRGB framebuffer
            fn imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
                let x = col[0].powf(2.2);
                let y = col[1].powf(2.2);
                let z = col[2].powf(2.2);
                let w = col[3]; //1.0 - (1.0 - col[3]).powf(2.2);
                [x, y, z, w]
            }

            let a = ((*color & 0xFF000000) >> 24) as f32 / 255.0;
            let r = ((*color & 0x00FF0000) >> 16) as f32 / 255.0;
            let g = ((*color & 0x0000FF00) >> 8) as f32 / 255.0;
            let b = (*color & 0x000000FF) as f32 / 255.0;

            let linear = imgui_gamma_to_linear([r, g, b, a]);

            let mut new_color = 0u32;
            new_color |= ((linear[3] * 255.0) as u32) << 24;
            new_color |= ((linear[0] * 255.0) as u32) << 16;
            new_color |= ((linear[1] * 255.0) as u32) << 8;
            new_color |= (linear[2] * 255.0) as u32;

            *color = new_color;
        }

        editor.get_style().colors[imnodes::ColorStyle::GridBackground as usize] = 0xFF090909;
        editor.get_style().colors[imnodes::ColorStyle::GridLine as usize] = 0xFF181818;

        editor
    }

    // Allows access to the ui without the caller needing to be aware of locking. A frame must be started
    pub fn with_ui<F>(
        &self,
        f: F,
    ) where
        F: FnOnce(&mut imgui::Ui, &mut imnodes::Context),
    {
        let mut inner = self.inner.lock().unwrap();

        if inner.ui.is_none() {
            log::warn!("Tried to use imgui ui when a frame was not started");
            return;
        }

        if let Some(ui) = inner.ui {
            unsafe {
                (f)(&mut *ui, &mut inner.imnodes_context);
            }
        }
    }

    // Get reference to the underlying font atlas. The ref will be valid as long as this object
    // is not destroyed
    #[allow(dead_code)]
    pub fn font_atlas_texture(&self) -> &imgui::FontAtlasTexture {
        let inner = self.inner.lock().unwrap();
        assert!(!inner.font_atlas_texture.is_null());
        unsafe { &*inner.font_atlas_texture }
    }

    fn take_ui(inner: &mut Inner) -> Option<Box<imgui::Ui<'static>>> {
        let mut ui = None;
        std::mem::swap(&mut inner.ui, &mut ui);

        if let Some(ui) = ui {
            return Some(unsafe { Box::from_raw(ui) });
        }

        None
    }

    // Start a new frame
    pub fn begin_frame(
        &self,
        window: &winit::window::Window,
    ) {
        let mut inner_mutex_guard = self.inner.lock().unwrap();
        let mut inner = &mut *inner_mutex_guard;

        // Drop the old Ui if it exists
        if inner.ui.is_some() {
            log::warn!("a frame is already in progress, starting a new one");
            ImguiManager::take_ui(&mut inner);
        }

        inner
            .platform
            .prepare_frame(inner.context.io_mut(), window)
            .unwrap();
        let ui = Box::new(inner.context.frame());

        inner.want_capture_keyboard = ui.io().want_capture_keyboard;
        inner.want_capture_mouse = ui.io().want_capture_mouse;
        inner.want_set_mouse_pos = ui.io().want_set_mouse_pos;
        inner.want_text_input = ui.io().want_text_input;

        // Remove the lifetime of the Ui
        let ui_ptr: *mut imgui::Ui = Box::into_raw(ui);
        #[allow(clippy::transmute_ptr_to_ptr)]
        let ui_ptr: *mut imgui::Ui<'static> = unsafe { std::mem::transmute(ui_ptr) };

        // Store it as a raw pointer
        inner.ui = Some(ui_ptr);
    }

    // Returns true if a frame has been started (and not ended)
    #[allow(dead_code)]
    pub fn is_frame_started(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.ui.is_some()
    }

    // Finishes the frame. Draw data becomes available via get_draw_data()
    pub fn render(
        &self,
        window: &winit::window::Window,
    ) {
        let mut inner = self.inner.lock().unwrap();

        if inner.ui.is_none() {
            log::warn!("render() was called but a frame was not started");
            return;
        }

        let ui = ImguiManager::take_ui(&mut inner);
        if let Some(ui) = ui {
            inner.platform.prepare_render(&ui, window);
            ui.render();
        } else {
            log::warn!("ui did not exist");
        }
    }

    // Returns draw data (render must be called first to end the frame)
    #[allow(dead_code)]
    pub fn draw_data(&self) -> Option<&imgui::DrawData> {
        let inner = self.inner.lock().unwrap();

        if inner.ui.is_some() {
            log::warn!("get_draw_data() was called but a frame is in progress");
            return None;
        }

        let draw_data = unsafe { imgui_sys::igGetDrawData() };
        if draw_data.is_null() {
            log::warn!("no draw data available");
            return None;
        }

        let draw_data = unsafe { &*(draw_data as *mut imgui::DrawData) };
        Some(draw_data)
    }

    #[allow(dead_code)]
    pub fn want_capture_keyboard(&self) -> bool {
        self.inner.lock().unwrap().want_capture_keyboard
    }

    #[allow(dead_code)]
    pub fn want_capture_mouse(&self) -> bool {
        self.inner.lock().unwrap().want_capture_mouse
    }

    #[allow(dead_code)]
    pub fn want_set_mouse_pos(&self) -> bool {
        self.inner.lock().unwrap().want_set_mouse_pos
    }

    #[allow(dead_code)]
    pub fn want_text_input(&self) -> bool {
        self.inner.lock().unwrap().want_text_input
    }
}