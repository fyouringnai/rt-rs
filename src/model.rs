use std::path::Path;

use glow::*;

use image::DynamicImage::{ImageLuma8, ImageLumaA8, ImageRgb8, ImageRgba8};
use tobj::{load_obj, GPU_LOAD_OPTIONS};

use crate::mesh::{Mesh, Texture, Vertex};
use crate::object::Object;
use crate::shader::Shader;
use crate::utils::MATERIAL::DIFFUSE;

#[derive(Default)]
pub struct Model {
    pub mesh: Vec<Mesh>,
    pub texture_loaded: Vec<Texture>,
    directory: String,
}

impl Model {
    pub unsafe fn new(gl: &Context, path: &str) -> Model {
        let mut model = Model::default();
        model.load_model(gl, path);
        model
    }

    pub unsafe fn draw(&self, gl: &Context, shader: &Shader) {
        for mesh in &self.mesh {
            mesh.draw(gl, shader);
        }
    }

    pub fn get_primitives(&self, primitives: &mut Vec<Object>) {
        let mut vertices = Vec::new();
        let mut vertices_num = 0;
        for (_i, mesh) in self.mesh.iter().enumerate() {
            for vertex in &mesh.vertices {
                let tex_coord = [vertex.tex_coord[0], vertex.tex_coord[1], 0.0];

                vertices.push(vertex.position);
                vertices.push(vertex.normal);
                vertices.push(tex_coord);
                if vertices_num % 3 == 2 {
                    let mut vertex = Vec::new();
                    for i in 0..3 {
                        vertex.push(vertices[(vertices_num - (2 - i)) as usize * 3]);
                        vertex.push(vertices[(vertices_num - (2 - i)) as usize * 3 + 1]);
                        vertex.push(vertices[(vertices_num - (2 - i)) as usize * 3 + 2]);
                    }
                    let triangle = Object::new_triangle(vertex, DIFFUSE);
                    primitives.push(triangle);
                }
                vertices_num += 1;
            }
        }
    }

    unsafe fn load_model(&mut self, gl: &Context, path: &str) {
        let path = Path::new(path);

        self.directory = path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_str()
            .unwrap()
            .into();

        let obj = load_obj(path, &GPU_LOAD_OPTIONS);
        let (models, materials) = obj.unwrap();
        let materials = materials.unwrap();

        for model in models {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;

            let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
            let indices = mesh.indices.clone();

            let (p, n, t) = (&mesh.positions, &mesh.normals, &mesh.texcoords);
            for i in 0..num_vertices {
                vertices.push(Vertex {
                    position: [p[i * 3], p[i * 3 + 1], p[i * 3 + 2]],
                    normal: [n[i * 3], n[i * 3 + 1], n[i * 3 + 2]],
                    tex_coord: [t[i * 2], t[i * 2 + 1]],
                    ..Vertex::default()
                })
            }
            let mut textures: Vec<Texture> = Vec::new();

            if let Some(material_id) = mesh.material_id {
                let material = &materials[material_id];

                if let Some(path) = &material.diffuse_texture {
                    let texture = self.load_material_texture(gl, path, "diffuse_texture");
                    textures.push(texture);
                }

                if let Some(path) = &material.specular_texture {
                    let texture = self.load_material_texture(gl, path, "specular_texture");
                    textures.push(texture);
                }

                if let Some(path) = &material.normal_texture {
                    let texture = self.load_material_texture(gl, path, "normal_texture");
                    textures.push(texture);
                }
            }
            self.mesh.push(Mesh::new(gl, vertices, textures, indices))
        }
    }

    fn load_material_texture(&mut self, gl: &Context, path: &str, type_name: &str) -> Texture {
        let texture = self.texture_loaded.iter().find(|t| t.path == path);
        if let Some(texture) = texture {
            return texture.clone();
        }
        let texture = Texture {
            id: unsafe { texture_from_file(gl, path, &self.directory) },
            type_: type_name.into(),
            path: path.into(),
        };
        self.texture_loaded.push(texture.clone());
        texture
    }
}
unsafe fn texture_from_file(gl: &Context, path: &str, directory: &str) -> NativeTexture {
    let file_path = format!("{}/{}", directory, path);

    let texture_id = load_texture(gl, file_path.as_str());

    texture_id
}

pub unsafe fn load_texture(gl: &Context, file_path: &str) -> NativeTexture {
    let texture_id = gl.create_texture().unwrap();

    gl.bind_texture(TEXTURE_2D, Some(texture_id));
    gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as i32);
    gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as i32);
    gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR_MIPMAP_LINEAR as i32);
    gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as i32);

    let img = image::open(&Path::new(file_path)).expect("Failed to load texture");
    let format = match img {
        ImageLuma8(_) => RED,
        ImageLumaA8(_) => RG,
        ImageRgb8(_) => RGB,
        ImageRgba8(_) => RGBA,
        _ => RGB,
    };
    let img = img.flipv();
    let data = img.clone().into_bytes();

    gl.tex_image_2d(
        TEXTURE_2D,
        0,
        format as i32,
        img.width() as i32,
        img.height() as i32,
        0,
        format,
        UNSIGNED_BYTE,
        Some(&data),
    );
    gl.generate_mipmap(TEXTURE_2D);

    gl.bind_texture(TEXTURE_2D, None);
    texture_id
}
