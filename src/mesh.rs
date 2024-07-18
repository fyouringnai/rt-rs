use crate::shader::Shader;
use bytemuck::{cast_slice, Pod, Zeroable};
use glow::*;
use std::ffi::CString;
use std::mem::size_of;

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
    pub vao: VertexArray,

    vbo: Buffer,
    ebo: Buffer,
}

impl Mesh {
    pub unsafe fn new(
        gl: &Context,
        vertices: Vec<Vertex>,
        textures: Vec<Texture>,
        indices: Vec<u32>,
    ) -> Mesh {
        let mut mesh = Mesh {
            vertices,
            textures,
            indices,
            vao: gl.create_vertex_array().unwrap(),
            vbo: gl.create_buffer().unwrap(),
            ebo: gl.create_buffer().unwrap(),
        };
        mesh.set_mesh(gl);

        mesh
    }

    pub unsafe fn draw(&self, gl: &Context, shader: &Shader) {
        let mut diffuse_nr = 0;
        let mut specular_nr = 0;
        let mut normal_nr = 0;
        let mut height_nr = 0;

        for (i, texture) in self.textures.iter().enumerate() {
            gl.active_texture(TEXTURE0 + i as u32);
            let name = &texture.type_;
            let number = match name.as_str() {
                "diffuse_texture" => {
                    diffuse_nr += 1;
                    diffuse_nr
                }
                "specular_texture" => {
                    specular_nr += 1;
                    specular_nr
                }
                "normal_texture" => {
                    normal_nr += 1;
                    normal_nr
                }
                "height_texture" => {
                    height_nr += 1;
                    height_nr
                }
                _ => panic!("unknown texture type"),
            };
            let sampler = CString::new(format!("{}{}", name, number)).unwrap();
            shader.set_int(&gl, sampler.to_str().unwrap(), i as i32);
            gl.bind_texture(TEXTURE_2D, Some(texture.id));
        }
        gl.bind_vertex_array(Some(self.vao));
        gl.draw_elements(TRIANGLES, self.indices.len() as i32, UNSIGNED_INT, 0);

        gl.bind_vertex_array(None);
        gl.active_texture(TEXTURE0);
    }

    unsafe fn set_mesh(&mut self, gl: &Context) {
        gl.bind_vertex_array(Some(self.vao));
        gl.bind_buffer(ARRAY_BUFFER, Some(self.vbo));
        gl.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(self.ebo));

        gl.buffer_data_u8_slice(ARRAY_BUFFER, cast_slice(&self.vertices), STATIC_DRAW);
        gl.buffer_data_u8_slice(ELEMENT_ARRAY_BUFFER, cast_slice(&self.indices), STATIC_DRAW);

        let size = size_of::<Vertex>() as i32;
        gl.vertex_attrib_pointer_f32(0, 3, FLOAT, false, size, 0);
        gl.vertex_attrib_pointer_f32(1, 3, FLOAT, false, size, (3 * size_of::<f32>()) as i32);
        gl.vertex_attrib_pointer_f32(2, 2, FLOAT, false, size, (6 * size_of::<f32>()) as i32);
        gl.vertex_attrib_pointer_f32(3, 3, FLOAT, false, size, (8 * size_of::<f32>()) as i32);
        gl.vertex_attrib_pointer_f32(4, 3, FLOAT, false, size, (11 * size_of::<f32>()) as i32);

        gl.enable_vertex_attrib_array(0);
        gl.enable_vertex_attrib_array(1);
        gl.enable_vertex_attrib_array(2);
        gl.enable_vertex_attrib_array(3);
        gl.enable_vertex_attrib_array(4);

        gl.bind_vertex_array(None);
    }
}
