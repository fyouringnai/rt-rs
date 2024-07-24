use bytemuck::{Pod, Zeroable};
use glow::*;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
    pub tangent: [f32; 3],
    pub bi_tangent: [f32; 3],
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
            bi_tangent: [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Clone)]
pub struct Texture {
    pub id: NativeTexture,
    pub type_: String,
    pub path: String,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub textures: Vec<Texture>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, textures: Vec<Texture>, indices: Vec<u32>) -> Mesh {
        let mesh = Mesh {
            vertices,
            textures,
            indices,
        };

        mesh
    }
}
