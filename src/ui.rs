use crate::global_model_state::BoneTree;
use egui::containers::panel::TopBottomSide;
use egui::{Color32, Style};
use PMXUtil::pmx_types::pmx_types::PMXBone;

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum TabKind {
    Bone,
    View,
    TextureView,
    Shader,
}
pub(crate) struct Tabs(pub TabKind);
impl Tabs {
    pub(crate) fn display_tabs(&mut self, ctx: &egui::CtxRef) {
        egui::TopBottomPanel::new(TopBottomSide::Top, "Tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.0, TabKind::Bone, "Bone");
                ui.selectable_value(&mut self.0, TabKind::View, "View");
                ui.selectable_value(&mut self.0, TabKind::TextureView, "uv");
                ui.selectable_value(&mut self.0, TabKind::Shader, "shader");
            })
        });
    }
    fn get_current_tab(&self) -> TabKind {
        self.0
    }
}
pub struct EguiTreeView {
    pub(crate) selected: i32,
    bone_data_source: Vec<PMXBone>,
    bone_tree: BoneTree,
}
impl EguiTreeView {
    pub fn from_bone_tree(tree: BoneTree, data_source: &[PMXBone]) -> Self {
        Self {
            selected: 0,
            bone_data_source: data_source.to_vec(),
            bone_tree: tree,
        }
    }
    pub fn display_tree(&mut self, ui: &mut egui::Ui) {
        display_in_collapsing_header(
            &self.bone_tree,
            ui,
            &mut self.selected,
            &self.bone_data_source,
            0,
        );
    }
}
fn display_in_collapsing_header(
    tree: &BoneTree,
    ui: &mut egui::Ui,
    select: &mut i32,
    data_source: &[PMXBone],
    indent_level: usize,
) {
    let name = if tree.id == -1 {
        "-1:Root".to_string()
    } else {
        format!(
            "{}{}:{}",
            "\t".repeat(indent_level),
            tree.id,
            &data_source[tree.id as usize].name
        )
    };

    egui::Frame::none().show(ui, |ui| {
        ui.vertical(|ui| {
            let label = egui::SelectableLabel::new(tree.id == *select, name);
            if ui.add(label).clicked() {
                *select=tree.id;
                println!("{} selected", tree.id);
            }
            for sub_tree in tree.child.values() {
                display_in_collapsing_header(sub_tree, ui, select, data_source, indent_level + 1);
            }
        });
    });
}
#[derive(Debug,Copy, Clone,Eq, PartialEq)]
pub enum Lang {
    English,
    Japanese,
}
pub struct EguiBoneParameterView<'a> {
    lang: Lang,
    bone: &'a mut PMXBone,
    rebuild_signal: &'a bool,
    after_physics:bool
}
impl<'a> EguiBoneParameterView<'a> {
    pub fn new(bone: &'a mut PMXBone, rebuild_signal: &'a mut bool) -> Self {
        Self {
            lang: Lang::Japanese,
            bone,
            rebuild_signal,
            after_physics: false
        }
    }
    pub fn display_ui(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("ボーン名");
                    match self.lang {
                        Lang::English => {
                            ui.text_edit_singleline(&mut self.bone.english_name);
                        }
                        Lang::Japanese => {
                            ui.text_edit_singleline(&mut self.bone.name);
                        }
                    }
                    ui.selectable_value(&mut self.lang, Lang::Japanese, "日");
                    ui.selectable_value(&mut self.lang, Lang::English, "英");
                    ui.label("変形階層");
                   // ui.radio_value()
                    //ui.checkbox();
                });
                ui.horizontal(|ui|{
                   ui.label("位置");

                });
                ui.horizontal(|ui|{
                    ui.label("親ボーン");

                })
            })
        });
    }
}
