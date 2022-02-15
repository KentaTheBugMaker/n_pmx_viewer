use egui::{Color32, CtxRef, Vec2};
use epi::App;
use std::cell::Cell;
use std::sync::Arc;

pub struct Models {
    model: Vec<Model>,
    current_model: Arc<Cell<usize>>,
}
impl App for Models {
    fn update(&mut self, ctx: &CtxRef, frame: &epi::Frame) {
        //we display models in bottom bar.
        let cm = self.current_model.clone();
        egui::TopBottomPanel::bottom("Model_selector").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.model
                    .iter_mut()
                    .enumerate()
                    .for_each(|(model_number, model)| {
                        let cm = cm.clone();
                        let mut button = egui::Button::new(&model.name);
                        if cm.get() == model_number {
                            button = button.fill(Color32::LIGHT_BLUE);
                        }
                        let response = ui.add(button);
                        if response.clicked() {
                            cm.set(model_number);
                        }
                        //create small view to display thumbnail
                        if response.hovered() {
                            let mut center = response.rect.center_top();
                            center.y -= 176.0;
                            center.x -= 176.0;
                            egui::Area::new("thumbnail")
                                .fixed_pos(center)
                                .show(ctx, |ui| {
                                    ui.image(model.thumbnail, Vec2::new(352.0, 176.0));
                                });
                        }
                    })
            });
        });
    }

    fn name(&self) -> &str {
        "models view"
    }
}
pub struct Model {
    /// we render thumbnail by wgpu.
    /// which render skins only.
    thumbnail: egui::TextureId,
    name: String,
}
