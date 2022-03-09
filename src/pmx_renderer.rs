use std::collections::HashMap;
use std::ops::Range;
use PMXUtil::reader::VerticesStage;

use bytemuck::{Pod, Zeroable};
use egui_wgpu_backend::wgpu::util::{BufferInitDescriptor, DeviceExt};
use egui_wgpu_backend::wgpu::{
    BufferUsages, Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use image::EncodableLayout;
use std::path::Path;

pub struct RenderResource {
    ///share all vertices in model.
    vertices: egui_wgpu_backend::wgpu::Buffer,
    ///share all indices in model.
    indices: egui_wgpu_backend::wgpu::Buffer,
    materials: Vec<Material>,
    textures: Vec<egui_wgpu_backend::wgpu::Texture>,
}

impl RenderResource {
    fn from_loader<R: std::io::Read>(
        loader: VerticesStage<R>,
        device: &egui_wgpu_backend::wgpu::Device,
        queue: &egui_wgpu_backend::wgpu::Queue,
        pmx_base_path: &Path,
    ) -> Self {
        let (mut vertices, loader) = loader.read();
        let vertices: Vec<Vertex> = vertices.drain(..).map(|v| v.into()).collect();
        let (mut faces, loader) = loader.read();
        let indices: Vec<u32> = faces.drain(..).fold(vec![], |mut buffer, face| {
            buffer.extend(face.vertices.iter().map(|i| *i as u32));
            buffer
        });
        let (texture_paths, loader) = loader.read();
        let textures = if let Some(base_path) = pmx_base_path.parent() {
            texture_paths
                .iter()
                .filter_map(|tex_path| {
                    let path = base_path.join(tex_path);
                    image::open(path).ok()
                })
                .map(|image| image.into_rgba8())
                .map(|pixels| {
                    let extent = pixels.dimensions();
                    device.create_texture_with_data(
                        queue,
                        &TextureDescriptor {
                            label: None,
                            size: Extent3d {
                                width: extent.0,
                                height: extent.1,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: TextureDimension::D2,
                            format: TextureFormat::Rgba8UnormSrgb,
                            usage: TextureUsages::TEXTURE_BINDING,
                        },
                        pixels.as_bytes(),
                    )
                })
                .collect()
        } else {
            vec![]
        };
        let (materials, _) = loader.read();
        let mut from = 0;
        let materials: Vec<_> = materials
            .iter()
            .map(|material| {
                let mat = Material {
                    index_buffer_range: from..material.num_face_vertices as u32,
                };
                from += material.num_face_vertices as u32;
                mat
            })
            .collect();
        let vertices = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices.as_slice()),
            usage: BufferUsages::VERTEX,
        });
        let indices = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices.as_slice()),
            usage: BufferUsages::INDEX,
        });

        Self {
            vertices,
            indices,
            materials,
            textures,
        }
    }
}
pub struct Material {
    index_buffer_range: Range<u32>,
}
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    pos: [f32; 4],
    uv: [f32; 2],
    norm: [f32; 3],
}
impl From<PMXUtil::types::Vertex> for Vertex {
    fn from(vertex: PMXUtil::types::Vertex) -> Self {
        Self {
            pos: [
                vertex.position[0],
                vertex.position[1],
                vertex.position[2],
                1.0,
            ],
            uv: vertex.uv,
            norm: vertex.norm,
        }
    }
}
