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
                *select = tree.id;
                println!("{} selected", tree.id);
            }
            for sub_tree in tree.child.values() {
                display_in_collapsing_header(sub_tree, ui, select, data_source, indent_level + 1);
            }
        });
    });
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Lang {
    English,
    Japanese,
}

pub struct EguiBoneView {
    pub(crate) bones: Vec<PMXBone>,
    pub(crate) current_displaying_bone: i32,
    pub(crate) bone_tree: BoneTree,
    pub(crate) lang: Lang,
}

impl EguiBoneView {
    pub fn display(&mut self, ui: &mut egui::Ui) {
        //lets create tree view
        egui::containers::SidePanel::left("Bone tree view").min_width(270.0).show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                display_in_collapsing_header(
                    &self.bone_tree,
                    ui,
                    &mut self.current_displaying_bone,
                    &self.bones,
                    0,
                )
            });
        });
        //lets create bone parameter view
        let mut cloned_bone = self
            .bones
            .get(self.current_displaying_bone as usize)
            .unwrap()
            .clone();
        let mut bone_flags = PMXBoneFlags::from(cloned_bone.boneflag);
        let mut rebuilt_tree = false;
        let mut update_bone = false;
        egui::containers::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("ボーン名");
                    if match self.lang {
                        Lang::English => ui.text_edit_singleline(&mut cloned_bone.english_name),
                        Lang::Japanese => ui.text_edit_singleline(&mut cloned_bone.name),
                    }
                    .changed()
                    {
                        update_bone = true;
                    }
                    ui.selectable_value(&mut self.lang, Lang::Japanese, "日");
                    ui.selectable_value(&mut self.lang, Lang::English, "英");
                    ui.label("変形階層");
                    let deform = egui::DragValue::new(&mut cloned_bone.deform_depth);
                    if ui.add(deform).changed() {
                        update_bone = true;
                    }
                    ui.checkbox(&mut bone_flags.deform_after_physics, "物理後");
                });
                ui.horizontal(|ui| {
                    ui.label("位置");
                });
                ui.horizontal(|ui| {
                    ui.label("親ボーン");
                    if ui
                        .add(egui::DragValue::new(&mut cloned_bone.parent))
                        .changed()
                    {
                        rebuilt_tree = true;
                    }
                    let parent_name=if cloned_bone.parent == -1 {
                        "-"
                    }else{
                        let parent=self.bones.get(cloned_bone.parent as usize).unwrap();
                        match self.lang{
                            Lang::English => {&parent.english_name}
                            Lang::Japanese => {&parent.name}
                        }
                    };
                    ui.label(parent_name);
                });
            })
        });
        //ボーン情報更新
        if update_bone {
            *self
                .bones
                .get_mut(self.current_displaying_bone as usize)
                .unwrap() = cloned_bone;
        }
        //親ボーンを変更したのでツリー組み立てなおし
        if rebuilt_tree{
            self.bone_tree = BoneTree::from_iter(self.bones.iter());
        }
    }
}
/// the rust friendly PMX Bone flag representation
#[derive(Clone, Copy)]
struct PMXBoneFlags {
    is_target_is_other_bone: bool,
    deform_after_physics: bool, //0x1000
    allow_rotate: bool,
    allow_translate: bool,
    flag1: bool,
}
impl PMXBoneFlags {
    fn none() -> Self {
        Self {
            is_target_is_other_bone: false,
            deform_after_physics: false,
            allow_rotate: false,
            allow_translate: false,
            flag1: false,
        }
    }
}
impl From<u16> for PMXBoneFlags {
    fn from(raw_bone_flag: u16) -> Self {
        let mut bone_flag = Self::none();
        if raw_bone_flag & PMXUtil::pmx_types::pmx_types::BONE_FLAG_DEFORM_AFTER_PHYSICS_MASK
            == PMXUtil::pmx_types::pmx_types::BONE_FLAG_DEFORM_AFTER_PHYSICS_MASK
        {
            bone_flag.deform_after_physics = true;
        }
        if raw_bone_flag & PMXUtil::pmx_types::pmx_types::BONE_FLAG_TARGET_SHOW_MODE_MASK
            == PMXUtil::pmx_types::pmx_types::BONE_FLAG_TARGET_SHOW_MODE_MASK
        {
            bone_flag.is_target_is_other_bone = true;
        }
        if raw_bone_flag & PMXUtil::pmx_types::pmx_types::BONE_FLAG_ALLOW_ROTATE_MASK
            == PMXUtil::pmx_types::pmx_types::BONE_FLAG_ALLOW_ROTATE_MASK
        {
            bone_flag.allow_rotate = true;
        }
        if raw_bone_flag & PMXUtil::pmx_types::pmx_types::BONE_FLAG_ALLOW_TRANSLATE_MASK
            == PMXUtil::pmx_types::pmx_types::BONE_FLAG_ALLOW_TRANSLATE_MASK
        {
            bone_flag.allow_translate = true;
        }

        bone_flag
    }
}
impl Into<u16> for PMXBoneFlags {
    fn into(self) -> u16 {
        0
    }
}
