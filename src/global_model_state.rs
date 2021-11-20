use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use PMXUtil::pmx_types::pmx_types::{PMXBone, PMXModelInfo};

///モデルの回転,平行移動,倍率を表す
///
/// scale -> rotate -> translate の順番で計算する
struct ModelTransform {
    scale: f32,
    translate: [f32; 3],
    rotate: [f32; 3],
}
///モデルのプラットフォーム固有のテクスチャ情報
struct PMXTexture {}
///
struct PMXPart {
    indices: egui_wgpu_backend::wgpu::Buffer,
}
///パーツ固有の情報を表す
/// これらはユニフォームバッファもしくはpush constantで渡す
struct PMXPartUniform {
    ///拡散光
    diffuse: [f32; 4],
    ///鏡面光
    specular: [f32; 3],
    ///鏡面係数
    specular_factor: f32,
    ///環境光
    ambient_color: [f32; 3],
}

///モデルについての情報を表す
#[derive(Debug)]
pub(crate) struct Model {
    ///モデル名
    name: String,
    ///ワールド変換情報
    // transform:ModelTransform,
    ///テクスチャ
    //   textures:Vec<PMXTexture>,
    ///パーツ
    //  parts:Vec<PMXPart>,
    ///ボーン木
    pub(crate) bone_tree: BoneTree,
}
#[derive(Debug)]
pub struct BoneTree {
    id: i32,
    child: BTreeMap<i32, BoneTree>,
}

impl BoneTree {
    pub fn new() -> Self {
        Self {
            id: -1,
            child: Default::default(),
        }
    }
    pub fn append(&mut self, child_index: i32, bone_info: PMXBone) {
        if self.id == bone_info.parent {
            //親IDが自分と一致したなら子供に入れる
            println!("{} : inserted", child_index);
            self.child.insert(
                child_index,
                BoneTree {
                    id: child_index,
                    child: Default::default(),
                },
            );
        } else {
            //子供の中からidと追加したい要素の親idが一致するものを探す
            if let Some((_, tree)) = self
                .child
                .iter_mut()
                .find(|(id, _)| **id == bone_info.parent)
            {
                println!("{} : inserted", child_index);
                tree.child.insert(
                    child_index,
                    BoneTree {
                        id: child_index,
                        child: Default::default(),
                    },
                );
            } else {
                //一致しない場合更に子供の中に持っているものがないか調べる
                let _: Vec<_> = self
                    .child
                    .values_mut()
                    .map(|tree| tree.append(child_index, bone_info.clone()))
                    .collect();
            }
        }
    }
    pub fn dump_tree(&self, indent_level: usize, data_source: &[PMXBone]) -> String {
        let name = if self.id == -1 {
            "Root"
        } else {
            &data_source[self.id as usize].name
        };
        let mut dump_text = format!("{}:{}\n", self.id, name);
        for sub_tree in self.child.values() {
            dump_text += &"\t".repeat(indent_level);
            dump_text += &sub_tree.dump_tree(indent_level + 1, data_source);
        }
        dump_text
    }
    pub fn display_in_collapsing_header(&self, ui: &mut egui::Ui, data_source: &[PMXBone]) {
        let name = if self.id == -1 {
            "Root"
        } else {
            &data_source[self.id as usize].name
        };
        egui::CollapsingHeader::new(name).default_open(true).show(ui,|ui|{
            for sub_tree in self.child.values() {
                sub_tree.display_in_collapsing_header(ui,data_source);
            }
        });
    }
}

impl Model {
    pub fn new(model_info: PMXModelInfo) -> Self {
        Self {
            name: model_info.name.clone(),
            /*
            transform: ModelTransform {
                scale: 1.0,
                translate: [0.0,0.0,0.0],
                rotate: [0.0,0.0,0.0]
            },
            textures: vec![],
            parts: vec![],*/
            bone_tree: BoneTree::new(),
        }
    }
    ///親のインデックスを見ながらツリーを作っていく
    ///
    pub fn load_bones(&mut self, bones: &[PMXBone]) {
        for (index, bone) in bones.iter().enumerate() {
            self.bone_tree.append(index as i32, bone.clone());
        }
    }
}
#[test]
fn test_load_bone() {
    let env = std::env::var("PMX_PATH").unwrap();
    println!("{:?}", env);
    let pmx = PMXUtil::pmx_loader::PMXLoader::open(env);
    let (model_info, loader) = pmx.read_pmx_model_info();
    let bones = loader
        .read_pmx_vertices()
        .1
        .read_pmx_faces()
        .1
        .read_texture_list()
        .1
        .read_pmx_materials()
        .1
        .read_pmx_bones()
        .0;
    let mut model = Model::new(model_info);
    model.load_bones(&bones);
    println!("{}", model.bone_tree.dump_tree(0, &bones));
}
