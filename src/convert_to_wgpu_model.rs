use PMXUtil::pmx_types::pmx_types::PMXVertex;
struct WGPUVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}
impl From<PMXVertex> for WGPUVertex {
    fn from(vertex: PMXVertex) -> Self {
        Self {
            position: vertex.position,
            normal: vertex.norm,
            uv: vertex.uv,
        }
    }
}
