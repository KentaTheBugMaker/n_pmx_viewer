use egui::{Color32, Response, Ui, Vec2, Widget};

pub struct Models {
    model: Vec<Model>,
}

impl Models {
    pub fn new() -> Self {
        Self { model: vec![] }
    }
    pub fn dummy() -> Self {
        Self {
            model: vec![
                Model {
                    thumbnail: Default::default(),
                    name: "ミライアカリ".to_string(),
                },
                Model {
                    thumbnail: Default::default(),
                    name: "初音ミク".to_string(),
                },
            ],
        }
    }
    pub fn new_model(&mut self, name: &str) {
        self.model.push(Model {
            thumbnail: Default::default(),
            name: name.to_string(),
        })
    }
}
pub struct ModelSelector<'a> {
    models: &'a Models,
    current_model: &'a mut usize,
}
impl<'a> ModelSelector<'a> {
    pub fn create_view(models: &'a Models, highlight: &'a mut usize) -> Self {
        Self {
            models,
            current_model: highlight,
        }
    }
}
impl<'a> Widget for ModelSelector<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            self.models
                .model
                .iter()
                .enumerate()
                .for_each(|(model_number, model)| {
                    let mut button = egui::Button::new(&model.name);
                    if *self.current_model == model_number {
                        button = button.fill(Color32::LIGHT_BLUE);
                    }
                    let response = ui.add(button);
                    if response.clicked() {
                        *self.current_model = model_number;
                    }
                    //create small view to display thumbnail

                    if response.hovered() {
                        let mut center = response.rect.center_top();

                        center.y -= 190.0;
                        center.x -= 176.0;
                        if center.x < 0.0 {
                            center.x = 10.0
                        }
                        egui::Window::new(&model.name)
                            .fixed_pos(center)
                            .title_bar(false)
                            .resizable(false)
                            .collapsible(false)
                            .show(ui.ctx(), |ui| {
                                ui.image(model.thumbnail, Vec2::new(352.0, 176.0));
                            });
                    }
                })
        })
        .response
    }
}

pub struct Model {
    /// we render thumbnail by wgpu.
    /// which render skins only.
    thumbnail: egui::TextureId,
    name: String,
}
