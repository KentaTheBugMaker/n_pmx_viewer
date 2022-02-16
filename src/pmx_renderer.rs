use std::ops::Range;
use std::collections::HashMap;
use PMXUtil::pmx_types::PMXVertex;

pub struct RenderResource{
    ///share all vertices in model.
    vertices:egui_wgpu_backend::wgpu::Buffer,
    ///share all indices in model.
    indices:egui_wgpu_backend::wgpu::Buffer,
    materials:Vec<Material>,
}
pub struct Material{

    index_buffer_range:Range<u32>,
    textures:HashMap<String,egui_wgpu_backend::wgpu::Texture>,

}
struct Vertex{
    pos:[f32;4],
    uv:[f32;2],
    norm:[f32;3],
}
impl From<PMXUtil::pmx_types::PMXVertex> for Vertex{
    fn from(vertex: PMXVertex) -> Self {
        Self{
            pos: [vertex.position[0],vertex.position[1],vertex.position[2],1.0,],
            uv: vertex.uv,
            norm: vertex.norm
        }
    }
}