mod convert_to_wgpu_model;
mod ui;
mod global_model_state;

use std::iter;
use std::time::Instant;

use crate::ui::{TabKind, Tabs};
use egui::{FontDefinitions, Vec2};
use egui_wgpu_backend::wgpu::CommandEncoderDescriptor;
use egui_wgpu_backend::{epi, wgpu, RenderPass, ScreenDescriptor};
use egui_winit::winit;
use epi::*;
use std::process::exit;
use winit::event::Event::*;
use winit::event_loop::ControlFlow;

const INITIAL_WIDTH: u32 = 1280;
const INITIAL_HEIGHT: u32 = 720;

/// A simple egui + wgpu + winit based example.
fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("egui-wgpu_winit example")
        .with_inner_size(winit::dpi::PhysicalSize {
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    // WGPU 0.11+ support force fallback (if HW implementation not supported), set it to true or false (optional).
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let (mut device, mut queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    ))
    .unwrap();

    let size = window.inner_size();
    let surface_format = surface.get_preferred_format(&adapter).unwrap();
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &surface_config);
    let mut tabs = Tabs(TabKind::View);
    // We use the egui_wgpu_backend crate as the render backend.
    let mut egui_rpass = RenderPass::new(&device, surface_format, 1);
    let mut integration = egui_winit::State::new(&window);
    let mut egui_ctx = egui::CtxRef::default();

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let input = integration.take_egui_input(&window);

            let output_frame = match surface.get_current_texture() {
                Ok(frame) => frame,
                Err(e) => {
                    eprintln!("Dropped frame with error: {}", e);
                    return;
                }
            };
            let output_view = output_frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            egui_ctx.begin_frame(input);
            egui::SidePanel::left("my_side_panel").show(&egui_ctx, |ui| {
                ui.heading("Hello World!");
                if ui.button("Quit").clicked() {}
            });
            tabs.display_tabs(&egui_ctx);
            egui::Window::new("window").fixed_size(Vec2::new(300.0, 200.0));
            let (output, shapes) = egui_ctx.end_frame();
            println!("{}", shapes.len());
            let meshes = egui_ctx.tessellate(shapes);
            egui_rpass.update_texture(&device, &queue, &egui_ctx.texture());
            egui_rpass.update_user_textures(&device, &queue);
            let screen_descriptor = ScreenDescriptor {
                physical_width: surface_config.width,
                physical_height: surface_config.height,
                scale_factor: window.scale_factor() as f32,
            };
            egui_rpass.update_buffers(&device, &queue, &meshes, &screen_descriptor);
            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("egui_renderpass"),
            });

            egui_rpass.execute(
                &mut encoder,
                &output_view,
                &meshes,
                &screen_descriptor,
                Some(wgpu::Color::BLACK),
            );
            let command = encoder.finish();
            queue.submit(iter::once(command));
            output_frame.present();
            *control_flow = winit::event_loop::ControlFlow::Poll;
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            winit::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            winit::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            winit::event::Event::WindowEvent { event, .. } => {
                if let winit::event::WindowEvent::Resized(physical_size) = event {
                    surface_config.width = size.width;
                    surface_config.height = size.height;
                    surface.configure(&device, &surface_config);
                }
                integration.on_event(&egui_ctx, &event);
                if integration.is_quit_event(&event) {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }

                window.request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            winit::event::Event::LoopDestroyed => exit(0),

            _ => window.request_redraw(),
        }
    });
}
