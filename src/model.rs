use cgmath::Vector3;
use glow::*;
use image::DynamicImage::{ImageLuma8, ImageLumaA8, ImageRgb8, ImageRgba8};
use std::path::Path;
use tobj::{load_obj, GPU_LOAD_OPTIONS};

use crate::mesh::{Mesh, Texture, Vertex};
use crate::object::Object;
use crate::shader::Shader;
use crate::utils::{trans, translated, translated_normal, MATERIAL};

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

    pub fn get_primitives(
        &self,
        primitives: &mut Vec<Object>,
        transform: &Vec<Vector3<f32>>,
        constant: f32,
        material: MATERIAL,
    ) {
        let mut vertices = Vec::new();
        let mut vertices_num = 0;
        let mut texture_index = [-1.0, -1.0, -1.0, -1.0];
        let model = trans(transform[0], transform[1], transform[2]);
        for (_i, mesh) in self.mesh.iter().enumerate() {
            for (_i, texture) in mesh.textures.iter().enumerate() {
                let name = &texture.type_;
                match name.as_str() {
                    "diffuse_texture" => {
                        texture_index[0] += 1.0;
                    }
                    "specular_texture" => {
                        texture_index[1] += 1.0;
                    }
                    "normal_texture" => {
                        texture_index[2] += 1.0;
                    }
                    "height_texture" => {
                        texture_index[3] += 1.0;
                    }
                    _ => {}
                };
            }
            for vertex in &mesh.vertices {
                let tex_coord = [vertex.tex_coord[0], vertex.tex_coord[1], 0.0];

                vertices.push(vertex.position);
                vertices.push(vertex.normal);
                vertices.push(tex_coord);
                if vertices_num % 3 == 2 {
                    let mut vertex = Vec::new();
                    for i in 0..3 {
                        vertex.push(translated(
                            &vertices[(vertices_num - (2 - i)) as usize * 3],
                            &model,
                        ));
                        vertex.push(translated_normal(
                            &vertices[(vertices_num - (2 - i)) as usize * 3 + 1],
                            &model,
                        ));
                        vertex.push(vertices[(vertices_num - (2 - i)) as usize * 3 + 2]);
                    }
                    let texture_index_temp = [texture_index[0], texture_index[1], texture_index[2]];
                    vertex.push(texture_index_temp);
                    vertex.push([texture_index[3], 0.0, 0.0]);
                    let triangle = Object::new_mesh(vertex, constant, material.clone());
                    primitives.push(triangle);
                }
                vertices_num += 1;
            }
        }
    }

    pub fn use_textures(&self, gl: &Context, shader: &Shader) {
        let mut diffuse_nr = -1;
        let mut specular_nr = -1;
        let mut normal_nr = -1;
        let mut height_nr = -1;

        for (i, texture) in self.texture_loaded.iter().enumerate() {
            unsafe {
                gl.active_texture(TEXTURE3 + i as u32);
                gl.bind_texture(TEXTURE_2D, Some(texture.id));
            }
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
                _ => {
                    panic!("unknown texture type");
                }
            };

            shader.set_int(gl, &format!("{}{}", name, number), (i + 3) as i32);
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
            self.mesh.push(Mesh::new(vertices, textures, indices))
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

    pub fn delete(&self, gl: &Context) {
        for texture in &self.texture_loaded {
            unsafe {
                gl.delete_texture(texture.id);
            }
        }
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
