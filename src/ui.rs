use egui::containers::panel::TopBottomSide;
#[derive(Copy, Clone)]
pub(crate) enum TabKind {
    View,
    TextureView,
    Shader,
}
pub(crate) struct Tabs(pub TabKind);
impl Tabs {
    pub(crate) fn display_tabs(&mut self, ctx: &egui::CtxRef) {
        egui::TopBottomPanel::new(TopBottomSide::Top, "Tabs").show(ctx, |Ui| {
            Ui.horizontal(|Ui| {
                if Ui.button("View").clicked() {
                    self.0 = TabKind::View
                }
                if Ui.button("UV").clicked() {
                    self.0 = TabKind::TextureView
                }
                if Ui.button("Shaders").clicked() {
                    self.0 = TabKind::Shader
                }
            })
        });
    }
    fn get_current_tab(&self) -> TabKind {
        self.0
    }
}
