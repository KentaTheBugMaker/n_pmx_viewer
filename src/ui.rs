use crate::global_model_state::BoneTree;
use egui::containers::panel::TopBottomSide;

use PMXUtil::types::{Bone, BoneFlags, Header, ModelInfo, Vertex, VertexWeight};

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum TabKind {
    Info,
    Vertex,
    Face,
    Material,
    Bone,
    Morph,
    Frame,
    RigidBody,
    Joint,
    SoftBody,
    View,
    TextureView,
    Shader,
}
pub(crate) struct Tabs(pub TabKind);
impl Tabs {
    pub(crate) fn display_tabs(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::new(TopBottomSide::Top, "Tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.0, TabKind::Info, "Info");
                ui.selectable_value(&mut self.0, TabKind::Vertex, "Vertex");
                ui.selectable_value(&mut self.0, TabKind::Face, "Face");
                ui.selectable_value(&mut self.0, TabKind::Material, "Material");
                ui.selectable_value(&mut self.0, TabKind::Bone, "Bone");
                ui.selectable_value(&mut self.0, TabKind::Morph, "Morph");
                ui.selectable_value(&mut self.0, TabKind::Frame, "Frame");
                ui.selectable_value(&mut self.0, TabKind::RigidBody, "RigidBody");
                ui.selectable_value(&mut self.0, TabKind::Joint, "Joint");
                ui.selectable_value(&mut self.0, TabKind::SoftBody, "SoftBody");
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
    data_source: &[Bone],
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
    pub(crate) bones: Vec<Bone>,
    pub(crate) current_displaying_bone: i32,
    pub(crate) bone_tree: BoneTree,
    pub(crate) lang: Lang,
}

impl EguiBoneView {
    pub fn new(bones: &[Bone]) -> Self {
        Self {
            bones: bones.to_vec(),
            current_displaying_bone: 0,
            bone_tree: BoneTree::from_iter(bones.iter()),
            lang: Lang::Japanese,
        }
    }
    pub fn display(&mut self, ui: &mut egui::Ui) {
        //lets create tree view
        egui::containers::SidePanel::left("Bone tree view")
            .min_width(270.0)
            .show_inside(ui, |ui| {
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

        let mut cloned_bone = self
            .bones
            .get(self.current_displaying_bone as usize)
            .unwrap()
            .clone();
        let mut rebuilt_tree = false;
        egui::containers::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("ボーン名");
                    match self.lang {
                        Lang::English => ui.text_edit_singleline(&mut cloned_bone.english_name),
                        Lang::Japanese => ui.text_edit_singleline(&mut cloned_bone.name),
                    };

                    ui.selectable_value(&mut self.lang, Lang::Japanese, "日");
                    ui.selectable_value(&mut self.lang, Lang::English, "英");
                    ui.label("変形階層");
                    let deform = egui::DragValue::new(&mut cloned_bone.deform_depth);
                    ui.add(deform);
                    ui.checkbox(&mut cloned_bone.physics_after_deform, "物理後");
                });
                ui.horizontal(|ui| {
                    ui.label("位置");
                    ui.add(egui::DragValue::new(&mut cloned_bone.position[0]));
                    ui.add(egui::DragValue::new(&mut cloned_bone.position[1]));
                    ui.add(egui::DragValue::new(&mut cloned_bone.position[2]));
                });
                ui.horizontal(|ui| {
                    ui.label("親ボーン");

                    rebuilt_tree = ui
                        .add(egui::DragValue::new(&mut cloned_bone.parent))
                        .changed();
                    let parent_name = if cloned_bone.parent < 0 {
                        "-"
                    } else {
                        let parent = self.bones.get(cloned_bone.parent as usize).unwrap();
                        match self.lang {
                            Lang::English => &parent.english_name,
                            Lang::Japanese => &parent.name,
                        }
                    };
                    ui.label(parent_name);
                });
            })
        });
        //ボーン情報更新
        *self
            .bones
            .get_mut(self.current_displaying_bone as usize)
            .unwrap() = cloned_bone;

        //親ボーンを変更したのでツリー組み立てなおし
        if rebuilt_tree {
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
impl From<BoneFlags> for PMXBoneFlags {
    fn from(flags: BoneFlags) -> Self {
        Self {
            is_target_is_other_bone: flags.contains(BoneFlags::CONNECT_TO_OTHER_BONE),
            deform_after_physics: flags.contains(BoneFlags::PHYSICS_AFTER_DEFORM),
            allow_rotate: flags.contains(BoneFlags::ROTATABLE),
            allow_translate: flags.contains(BoneFlags::TRANSLATABLE),
            flag1: false,
        }
    }
}

impl From<PMXBoneFlags> for u16 {
    fn from(_: PMXBoneFlags) -> Self {
        0
    }
}

pub struct PMXInfoView {
    pub(crate) header: Header,
    pub(crate) model_info: ModelInfo,
    pub(crate) encode: Encode,
    pub(crate) lang: Lang,
    additonal_uvs_changed: bool,
}
impl PMXInfoView {
    pub fn new(header: Header, model_info: ModelInfo) -> Self {
        let encode = header.encode.into();
        Self {
            header,
            model_info,
            encode,
            lang: Lang::Japanese,
            additonal_uvs_changed: false,
        }
    }
    pub(crate) fn display(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label("System");
                egui::Frame::none().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("PMX Version : {:?}", self.header.version));
                        ui.label("character encoding :");
                        egui::ComboBox::from_label("")
                            .selected_text(self.encode.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.encode,
                                    Encode::UTF16LE,
                                    Encode::UTF16LE.to_string(),
                                );
                                ui.selectable_value(
                                    &mut self.encode,
                                    Encode::UTF8,
                                    Encode::UTF8.to_string(),
                                );
                            });
                        ui.label("additional uvs");
                        let saved_addtional_uvs = self.header.additional_uv;
                        egui::ComboBox::from_label("uvs")
                            .selected_text(self.header.additional_uv.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.header.additional_uv, 0, "0");
                                ui.selectable_value(&mut self.header.additional_uv, 1, "1");
                                ui.selectable_value(&mut self.header.additional_uv, 2, "2");
                                ui.selectable_value(&mut self.header.additional_uv, 3, "3");
                                ui.selectable_value(&mut self.header.additional_uv, 4, "4");
                            });
                        if saved_addtional_uvs != self.header.additional_uv {
                            self.additonal_uvs_changed = true;
                        }
                    });
                });

                egui::Frame::none().show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("model name");
                            let name = match self.lang {
                                Lang::English => &mut self.model_info.name_en,
                                Lang::Japanese => &mut self.model_info.name,
                            };
                            ui.text_edit_singleline(name);
                            ui.selectable_value(&mut self.lang, Lang::Japanese, "日");
                            ui.selectable_value(&mut self.lang, Lang::English, "英");
                        });
                        ui.label("comment");
                        let comment = match self.lang {
                            Lang::English => &mut self.model_info.comment_en,
                            Lang::Japanese => &mut self.model_info.comment,
                        };
                        ui.text_edit_multiline(comment);
                    })
                });
            });
        });
    }
    pub fn query_updated_header(&mut self) -> Option<Header> {
        if self.additonal_uvs_changed {
            self.additonal_uvs_changed = false;
            Some(self.header.clone())
        } else {
            None
        }
    }
}
#[derive(Eq, PartialEq, Copy, Clone)]
pub(crate) enum Encode {
    UTF16LE,
    UTF8,
}
impl From<PMXUtil::types::Encode> for Encode {
    fn from(encode: PMXUtil::types::Encode) -> Self {
        match encode {
            PMXUtil::types::Encode::UTF8 => Self::UTF8,
            PMXUtil::types::Encode::Utf16Le => Self::UTF16LE,
        }
    }
}
impl ToString for Encode {
    fn to_string(&self) -> String {
        match self {
            Encode::UTF16LE => "UTF-16(LE)".to_string(),
            Encode::UTF8 => "UTF-8".to_string(),
        }
    }
}
pub struct PMXVertexView {
    vertices: Vec<Vertex>,
    selected: usize,
    display_sdef_parameter: bool,
    update_vertices: bool,
    header: Header,
    bones: Vec<Bone>,
    selected_uv: u8,
    lang: Lang,
}
impl PMXVertexView {
    pub fn new(vertices: Vec<Vertex>, header: Header, bones: &[Bone]) -> Self {
        Self {
            vertices,
            selected: 0,
            display_sdef_parameter: false,
            update_vertices: true,
            header,
            bones: Vec::from(bones),
            selected_uv: 0,
            lang: Lang::Japanese,
        }
    }
    pub fn update_header(&mut self, header: Header) {
        self.header = header;
    }
    pub fn update_bone(&mut self, bones: &[Bone]) {
        self.bones = bones.to_vec();
    }
    pub fn display(&mut self, ui: &mut egui::Ui) {
        egui::SidePanel::left("Vertices").show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, vertices) in self.vertices.iter().enumerate() {
                    if ui
                        .add(egui::SelectableLabel::new(
                            self.selected == index,
                            format!("{}: {:?}", index, vertices.position),
                        ))
                        .clicked()
                    {
                        self.selected = index;
                    }
                }
            });
        });
        let mut cloned_vertex = self.vertices[self.selected].clone();
        let mut weight_kind: WeightKind = cloned_vertex.weight_type.into();
        let mut weight_parameters: WeightParameters = cloned_vertex.weight_type.into();
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    //position and normal
                    egui::Frame::none().show(ui, |ui| {
                        ui.vertical(|ui| {
                            //position
                            ui.horizontal(|ui| {
                                ui.label("position x:");
                                ui.add(egui::DragValue::new(&mut cloned_vertex.position[0]));
                                ui.label("y:");
                                ui.add(egui::DragValue::new(&mut cloned_vertex.position[1]));
                                ui.label("z:");
                                ui.add(egui::DragValue::new(&mut cloned_vertex.position[2]));
                            });
                            //normal
                            ui.horizontal(|ui| {
                                ui.label("normal x:");
                                ui.add(egui::DragValue::new(&mut cloned_vertex.norm[0]));
                                ui.label("y:");
                                ui.add(egui::DragValue::new(&mut cloned_vertex.norm[1]));
                                ui.label("z:");
                                ui.add(egui::DragValue::new(&mut cloned_vertex.norm[2]));
                            });
                        });
                    });
                    //edge mag
                    ui.label("edge magnifier");
                    ui.add(egui::DragValue::new(&mut cloned_vertex.edge_mag));
                });
                //uv
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("UV: u");
                        ui.add(egui::DragValue::new(&mut cloned_vertex.uv[0]));
                        ui.label("v");
                        ui.add(egui::DragValue::new(&mut cloned_vertex.uv[1]));
                        ui.add(egui::Label::new("※追加UVの有効数設定はInfoから設定"));
                    });
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_source("AdditionalUV select").show_ui(ui, |ui| {
                            for i in 0..self.header.additional_uv {
                                ui.selectable_value(
                                    &mut self.selected_uv,
                                    i,
                                    format!("Addtional UV {}", i),
                                );
                            }
                        });
                        ui.label("x");
                        ui.add(egui::DragValue::new(
                            &mut cloned_vertex.add_uv[self.selected_uv as usize][0],
                        ));
                        ui.label("y");
                        ui.add(egui::DragValue::new(
                            &mut cloned_vertex.add_uv[self.selected_uv as usize][1],
                        ));
                        ui.label("z");
                        ui.add(egui::DragValue::new(
                            &mut cloned_vertex.add_uv[self.selected_uv as usize][2],
                        ));
                        ui.label("w");
                        ui.add(egui::DragValue::new(
                            &mut cloned_vertex.add_uv[self.selected_uv as usize][3],
                        ));
                    });
                });
                //bone weight
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            egui::ComboBox::from_label("変形方式")
                                .selected_text(weight_kind.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut weight_kind,
                                        WeightKind::BDEF1,
                                        "BDEF1",
                                    );
                                    ui.selectable_value(
                                        &mut weight_kind,
                                        WeightKind::BDEF2,
                                        "BDEF2",
                                    );
                                    ui.selectable_value(
                                        &mut weight_kind,
                                        WeightKind::BDEF4,
                                        "BDEF4",
                                    );
                                    ui.selectable_value(&mut weight_kind, WeightKind::Sdef, "SDEF");
                                    ui.selectable_value(&mut weight_kind, WeightKind::Qdef, "QDEF");
                                });
                        });
                        ui.vertical(|ui| {
                            ui.add(egui::DragValue::new(&mut weight_parameters.bone_indices[0]));
                            ui.add(egui::DragValue::new(&mut weight_parameters.bone_indices[1]));
                            ui.add(egui::DragValue::new(&mut weight_parameters.bone_indices[2]));
                            ui.add(egui::DragValue::new(&mut weight_parameters.bone_indices[3]));
                        });
                        ui.vertical(|ui| {
                            ui.add(egui::DragValue::new(&mut weight_parameters.weights[0]));
                            ui.add(egui::DragValue::new(&mut weight_parameters.weights[1]));
                            ui.add(egui::DragValue::new(&mut weight_parameters.weights[2]));
                            ui.add(egui::DragValue::new(&mut weight_parameters.weights[3]));
                        });
                        let fetch_bone_name = |index: i32| -> &str {
                            match self.bones.get(index as usize) {
                                None => "-",
                                Some(bone) => match self.lang {
                                    Lang::English => &bone.english_name,
                                    Lang::Japanese => &bone.name,
                                },
                            }
                        };
                        ui.vertical(|ui| {
                            ui.label(fetch_bone_name(weight_parameters.bone_indices[0]));
                            ui.label(fetch_bone_name(weight_parameters.bone_indices[1]));
                            ui.label(fetch_bone_name(weight_parameters.bone_indices[2]));
                            ui.label(fetch_bone_name(weight_parameters.bone_indices[3]));
                        })
                    });
                })
            });
        });
    }
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum WeightKind {
    BDEF1,
    BDEF2,
    BDEF4,
    Sdef,
    Qdef,
}
impl ToString for WeightKind {
    fn to_string(&self) -> String {
        match self {
            WeightKind::BDEF1 => "BDEF1",
            WeightKind::BDEF2 => "BDEF2",
            WeightKind::BDEF4 => "BDEF4",
            WeightKind::Sdef => "SDEF",
            WeightKind::Qdef => "QDEF",
        }
        .to_string()
    }
}
impl From<VertexWeight> for WeightKind {
    fn from(weight: VertexWeight) -> Self {
        match weight {
            VertexWeight::BDEF1(_) => WeightKind::BDEF1,
            VertexWeight::BDEF2 { .. } => WeightKind::BDEF2,
            VertexWeight::BDEF4 { .. } => WeightKind::BDEF4,
            VertexWeight::SDEF { .. } => WeightKind::Sdef,
            VertexWeight::QDEF { .. } => WeightKind::Qdef,
        }
    }
}
struct WeightParameters {
    weights: [f32; 4],
    bone_indices: [i32; 4],
}
impl From<VertexWeight> for WeightParameters {
    fn from(weight: VertexWeight) -> Self {
        match weight {
            VertexWeight::BDEF1(x) => Self {
                weights: [1.0, 0.0, 0.0, 0.0],
                bone_indices: [x, -1, -1, -1],
            },
            VertexWeight::BDEF2 {
                bone_index_1,
                bone_index_2,
                bone_weight_1,
            } => Self {
                weights: [bone_weight_1, 1.0 - bone_weight_1, 0.0, 0.0],
                bone_indices: [bone_index_1, bone_index_2, -1, -1],
            },
            VertexWeight::BDEF4 {
                bone_index_1,
                bone_index_2,
                bone_index_3,
                bone_index_4,
                bone_weight_1,
                bone_weight_2,
                bone_weight_3,
                bone_weight_4,
            } => Self {
                weights: [bone_weight_1, bone_weight_2, bone_weight_3, bone_weight_4],
                bone_indices: [bone_index_1, bone_index_2, bone_index_3, bone_index_4],
            },
            VertexWeight::SDEF {
                bone_index_1,
                bone_index_2,
                bone_weight_1,
                sdef_c: _,
                sdef_r0: _,
                sdef_r1: _,
            } => Self {
                weights: [bone_weight_1, 1.0 - bone_weight_1, 0.0, 0.0],
                bone_indices: [bone_index_1, bone_index_2, -1, -1],
            },
            VertexWeight::QDEF {
                bone_index_1,
                bone_index_2,
                bone_index_3,
                bone_index_4,
                bone_weight_1,
                bone_weight_2,
                bone_weight_3,
                bone_weight_4,
            } => Self {
                weights: [bone_weight_1, bone_weight_2, bone_weight_3, bone_weight_4],
                bone_indices: [bone_index_1, bone_index_2, bone_index_3, bone_index_4],
            },
        }
    }
}
