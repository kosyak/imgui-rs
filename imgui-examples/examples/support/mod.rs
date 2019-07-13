use glium::glutin::{self, event::{ElementState, VirtualKeyCode, Event, WindowEvent}, platform::unix::WindowExtUnix};
use glium::{Display, Surface};
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;

// mod clipboard;

pub struct System {
    pub event_loop: glutin::event_loop::EventLoop<()>,
    pub display: Display,
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub font_size: f32,
}

pub fn init(title: &str) -> System {
    let _title = match title.rfind('/') {
        Some(idx) => title.split_at(idx + 1).1,
        None => title,
    };
    let event_loop = glutin::event_loop::EventLoop::new();
    let cb = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::GlThenGles { opengl_version: (3, 0), opengles_version: (2, 0) })
        .with_vsync(true);
    let wb = glutin::window::WindowBuilder::new()
        .with_fullscreen(Some(event_loop.primary_monitor()));
    let gl_window = cb.build_windowed(wb, &event_loop).unwrap();
    let display = Display::with_debug(gl_window, glium::debug::DebugCallbackBehavior::PrintAll).unwrap();

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    // if let Some(backend) = clipboard::init() {
    //     imgui.set_clipboard_backend(Box::new(backend));
    // } else {
    //     eprintln!("Failed to initialize clipboard");
    // }

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
    }

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (48.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        },
        FontSource::TtfData {
            data: include_bytes!("../../../resources/mplus-1p-regular.ttf"),
            // data: include_bytes!("../../../resources/Roboto-Regular.ttf"),
            size_pixels: font_size,
            config: Some(FontConfig {
                rasterizer_multiply: 1.75,
                glyph_ranges: FontGlyphRanges::japanese(),
                ..FontConfig::default()
            }),
        },
    ]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
    imgui.io_mut().config_flags |= imgui::ConfigFlags::NAV_ENABLE_GAMEPAD;

    let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    System {
        event_loop,
        display,
        imgui,
        platform,
        renderer,
        font_size,
    }
}

impl System {
    pub fn main_loop<F: FnMut(&mut bool, &mut Ui)>(self, mut run_ui: F) {
        let System {
            event_loop,
            display,
            mut imgui,
            platform,
            mut renderer,
            ..
        } = self;
        let gl_window = display.gl_window();
        let window = gl_window.window();
        let mut last_frame = Instant::now();
        let mut run = true;

        while run {
            // event_loop.poll_events(|event| {
            //     platform.handle_event(imgui.io_mut(), &window, &event);

            //     if let Event::WindowEvent { event, .. } = event {
            //         if let WindowEvent::CloseRequested = event {
            //             run = false;
            //         }
            //     }
            // });

            let io = imgui.io_mut();
            platform
                .prepare_frame(io, &window)
                .expect("Failed to start frame");
            last_frame = io.update_delta_time(last_frame);
            let mut ui = imgui.frame();
            run_ui(&mut run, &mut ui);

            let mut target = display.draw();
            target.clear_color_srgb(0.1, 0.1, 0.1, 1.0);
            platform.prepare_render(&ui, &window);
            let draw_data = ui.render();
            renderer
                .render(&mut target, draw_data)
                .expect("Rendering failed");

            target.finish().expect("Failed to swap buffers");
            window.gbm_page_flip();
        }
    }
}
