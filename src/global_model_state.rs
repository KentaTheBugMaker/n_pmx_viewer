use std::collections::BTreeMap;
use std::fmt::Debug;

use PMXUtil::types::{Bone, ModelInfo};

///モデルについての情報を表す
#[derive(Debug)]
pub(crate) struct Model {
    ///モデル名
    name: String,
    ///ボーン木
    pub(crate) bone_tree: Option<BoneTree>,
}
#[derive(Debug)]
pub struct BoneTree {
    pub(crate) id: i32,
    pub(crate) child: BTreeMap<i32, BoneTree>,
}

impl BoneTree {
    pub fn from_iter<'a>(iter: impl Iterator<Item = &'a Bone> + Clone) -> Self {
        let root = iter
            .clone()
            .enumerate()
            .find(|(_, bone)| bone.parent == -1)
            .unwrap();
        let mut root_node = BoneTree {
            id: root.0 as i32,
            child: Default::default(),
        };
        for (index, bone) in iter.enumerate() {
            root_node.append(index as i32, bone.clone());
        }
        root_node
    }
    pub fn append(&mut self, child_index: i32, bone_info: Bone) {
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
    pub fn dump_tree(&self, indent_level: usize, data_source: &[Bone]) -> String {
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
}
impl Model {
    pub fn new(model_info: ModelInfo) -> Self {
        Self {
            name: model_info.name.clone(),
            bone_tree: None,
        }
    }
    ///親のインデックスを見ながらツリーを作っていく
    ///
    pub fn load_bones(&mut self, bones: &[Bone]) {
        let root = bones
            .iter()
            .enumerate()
            .find(|(_, bone)| bone.parent == -1)
            .unwrap();
        let mut root_node = BoneTree {
            id: root.0 as i32,
            child: Default::default(),
        };
        for (index, bone) in bones.iter().enumerate() {
            root_node.append(index as i32, bone.clone());
        }
        self.bone_tree.replace(root_node);
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
