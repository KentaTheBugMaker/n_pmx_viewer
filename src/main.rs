mod global_model_state;
mod model_selector;
mod pmx_renderer;
mod ui;

use std::iter;

use crate::ui::{EguiBoneView, PMXInfoView, PMXVertexView, TabKind, Tabs};

use egui_wgpu_backend::wgpu::CommandEncoderDescriptor;
use egui_wgpu_backend::{wgpu, RenderPass, ScreenDescriptor};
use egui_winit::winit;

use crate::model_selector::ModelSelector;
use egui::{FontData, FullOutput};
use egui_winit::winit::event::WindowEvent;
use egui_winit::winit::event_loop::ControlFlow;
use std::io::Read;
use std::process::exit;
use std::sync::{Arc, RwLock};
use PMXUtil::reader::ModelInfoStage;

const INITIAL_WIDTH: u32 = 1280;
const INITIAL_HEIGHT: u32 = 720;

fn create_new_model_tab<R: Read>(
    pmx: ModelInfoStage<R>,
) -> (String, (PMXInfoView, PMXVertexView, EguiBoneView, Tabs)) {
    let header = pmx.get_header();
    let (model_info, loader) = pmx.read();
    let (vertices, loader) = loader.read();
    let (bones, loader) = loader.read().1.read().1.read().1.read();
    let pmx_info_view = PMXInfoView::new(header.clone(), model_info.clone());
    let pmx_vertex_view = PMXVertexView::new(vertices, header, &bones);
    let bone_view = EguiBoneView::new(&bones);
    (
        model_info.name,
        (
            pmx_info_view,
            pmx_vertex_view,
            bone_view,
            Tabs(TabKind::Info),
        ),
    )
}

/// A simple egui + wgpu + winit based example.
fn main() {
    let mut model_data_views: Vec<(PMXInfoView, PMXVertexView, EguiBoneView, Tabs)> = Vec::new();

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

    let (device, queue) = pollster::block_on(adapter.request_device(
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
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &surface_config);
    let models = Arc::new(RwLock::new(model_selector::Models::new()));
    // We use the egui_wgpu_backend crate as the render backend.
    let mut egui_rpass = RenderPass::new(&device, surface_format, 1);
    let mut integration =
        egui_winit::State::new(device.limits().max_texture_dimension_2d as usize, &window);

    let egui_ctx = egui::Context::default();

    let mut fonts = egui::FontDefinitions::default();

    if let Ok(noto_jp_bytes) = std::fs::read("./resources/NotoSansJP-Regular.otf") {
        fonts
            .font_data
            .insert("JP".to_owned(), FontData::from_owned(noto_jp_bytes));
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "JP".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "JP".to_owned());
        egui_ctx.set_fonts(fonts);
    }

    let mut model_number = 0;
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
            if let Some(model_data_view) = model_data_views.get_mut(model_number) {
                model_data_view.3.display_tabs(&egui_ctx);

                egui::CentralPanel::default().show(&egui_ctx, |ui| match model_data_view.3 .0 {
                    TabKind::Info => {
                        model_data_view.0.display(ui);
                    }
                    TabKind::Vertex => {
                        model_data_view.1.display(ui);
                    }
                    TabKind::Bone => {
                        model_data_view.2.display(ui);
                    }

                    TabKind::View => {}
                    TabKind::TextureView => {}
                    TabKind::Shader => {}
                    _ => {}
                });
                if let Some(header) = model_data_view.0.query_updated_header() {
                    model_data_view.1.update_header(header)
                }
            }
            egui::TopBottomPanel::bottom("model_selector").show(&egui_ctx, |ui| {
                ui.add(ModelSelector::create_view(
                    &models.read().unwrap(),
                    &mut model_number,
                ))
            });

            let full_output = egui_ctx.end_frame();
            let FullOutput {
                platform_output,
                needs_repaint,
                textures_delta,
                shapes,
            } = full_output;
            egui_rpass
                .add_textures(&device, &queue, &textures_delta)
                .ok();

            let meshes = egui_ctx.tessellate(shapes);
            let screen_descriptor = ScreenDescriptor {
                physical_width: surface_config.width,
                physical_height: surface_config.height,
                scale_factor: window.scale_factor() as f32,
            };
            egui_rpass.update_buffers(&device, &queue, &meshes, &screen_descriptor);
            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("egui_renderpass"),
            });

            egui_rpass
                .execute(
                    &mut encoder,
                    &output_view,
                    &meshes,
                    &screen_descriptor,
                    Some(wgpu::Color::BLACK),
                )
                .ok();
            egui_rpass.remove_textures(textures_delta).ok();
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
                match event {
                    WindowEvent::Resized(physical_size) => {
                        surface_config.width = physical_size.width;
                        surface_config.height = physical_size.height;
                        surface.configure(&device, &surface_config);
                    }
                    WindowEvent::Moved(_) => {}
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Destroyed => {}
                    WindowEvent::DroppedFile(ref file) => {
                        if file.extension().and_then(|path| path.to_str()) == Some("zip") {
                            #[cfg(not(target_arch = "wasm32"))]
                            let reader = std::fs::File::open(&file).ok();
                            if let Some(reader) = reader {
                                let zip_ar = zip::read::ZipArchive::new(reader).ok();
                                if let Some(mut ar) = zip_ar {
                                    let mut pmx_path = None;
                                    for name in ar.file_names() {
                                        println!("zip_content {}", name);
                                        if name.contains("pmx") {
                                            pmx_path = Some(name.to_owned());
                                        }
                                    }

                                    if let Some(pmx_path) = pmx_path {
                                        if let Some(pmx_file) = ar.by_name(&pmx_path).ok() {
                                            if let Some(reader) =
                                                PMXUtil::reader::ModelInfoStage::from_reader(
                                                    pmx_file,
                                                )
                                            {
                                                let (name, data) = create_new_model_tab(reader);
                                                models
                                                    .write()
                                                    .map(|mut models| models.new_model(&name))
                                                    .ok();
                                                model_data_views.push(data);
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                        }
                    }
                    WindowEvent::HoveredFile(_) => {}
                    WindowEvent::HoveredFileCancelled => {}
                    WindowEvent::ReceivedCharacter(_) => {}
                    WindowEvent::Focused(_) => {}
                    WindowEvent::KeyboardInput { .. } => {}
                    WindowEvent::ModifiersChanged(_) => {}
                    WindowEvent::CursorMoved { .. } => {}
                    WindowEvent::CursorEntered { .. } => {}
                    WindowEvent::CursorLeft { .. } => {}
                    WindowEvent::MouseWheel { .. } => {}
                    WindowEvent::MouseInput { .. } => {}
                    WindowEvent::TouchpadPressure { .. } => {}
                    WindowEvent::AxisMotion { .. } => {}
                    WindowEvent::Touch(_) => {}
                    WindowEvent::ScaleFactorChanged { .. } => {}
                    WindowEvent::ThemeChanged(_) => {}
                }

                integration.on_event(&egui_ctx, &event);

                window.request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            winit::event::Event::LoopDestroyed => exit(0),

            _ => window.request_redraw(),
        }
    });
}
