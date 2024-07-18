use std::ffi::CString;

use bytemuck::cast_slice;
use glow::{
    Context, FLOAT, HasContext, NEAREST, NO_ERROR, REPEAT, RGB, RGB32F, Texture,
    TEXTURE0, TEXTURE1, TEXTURE2, TEXTURE3, TEXTURE4, TEXTURE_2D, TEXTURE_MAG_FILTER,
    TEXTURE_MIN_FILTER, TEXTURE_WRAP_S, TEXTURE_WRAP_T,
};

use crate::model::Model;
use crate::shader::Shader;

pub struct ModelTexture {
    pub position_texture: Texture,
    pub normal_texture: Texture,
    pub texcoord_texture: Texture,
    pub vertices_num: i32,
}

impl ModelTexture {
    pub fn new(gl: &Context) -> ModelTexture {
        let model_texture = ModelTexture {
            position_texture: unsafe { gl.create_texture().unwrap() },
            normal_texture: unsafe { gl.create_texture().unwrap() },
            texcoord_texture: unsafe { gl.create_texture().unwrap() },
            vertices_num: 0,
        };
        unsafe {
            assert_eq!(gl.get_error(), NO_ERROR);
        }

        model_texture
    }

    pub fn set_texture(&mut self, gl: &Context, model: &Model) {
        let mut position = Vec::new();
        let mut normal = Vec::new();
        let mut texcoord = Vec::new();

        for (i, mesh) in model.mesh.iter().enumerate() {
            for vertex in &mesh.vertices {
                position.push(vertex.position);
                normal.push(vertex.normal);
                let mut tex_coord = [vertex.tex_coord[0], vertex.tex_coord[1], 0.0];
                for texture in &mesh.textures {
                    let name = &texture.type_;
                    match name.as_str() {
                        "diffuse_texture" => {
                            tex_coord[2] = (4 * i) as f32;
                        }
                        "specular_texture" => {
                            tex_coord[2] = (4 * i + 1) as f32;
                        }
                        "normal_texture" => {
                            tex_coord[2] = (4 * i + 2) as f32;
                        }
                        "height_texture" => {
                            tex_coord[2] = (4 * i + 3) as f32;
                        }
                        _ => panic!("unknown texture type"),
                    };
                }
                texcoord.push(tex_coord);
                self.vertices_num += 1;
            }
        }
        unsafe {
            gl.bind_texture(TEXTURE_2D, Some(self.position_texture));
            gl.tex_image_2d(
                TEXTURE_2D,
                0,
                RGB32F as i32,
                2048,
                2048,
                0,
                RGB,
                FLOAT,
                Some(cast_slice(&position)),
            );

            assert_eq!(gl.get_error(), NO_ERROR);

            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as i32);

            gl.bind_texture(TEXTURE_2D, Some(self.normal_texture));
            gl.tex_image_2d(
                TEXTURE_2D,
                0,
                RGB32F as i32,
                2048,
                2048,
                0,
                RGB,
                FLOAT,
                Some(cast_slice(&normal)),
            );
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as i32);

            gl.bind_texture(TEXTURE_2D, Some(self.texcoord_texture));
            gl.tex_image_2d(
                TEXTURE_2D,
                0,
                RGB32F as i32,
                2048,
                2048,
                0,
                RGB,
                FLOAT,
                Some(cast_slice(&texcoord)),
            );
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as i32);

            gl.bind_texture(TEXTURE_2D, None);
        }
    }

    pub fn use_texture(&self, gl: &Context, model: &Model, shader: &Shader) {
        unsafe {
            shader.set_int(&gl, "verticesNum", self.vertices_num);
            gl.active_texture(TEXTURE1);
            gl.bind_texture(TEXTURE_2D, Some(self.position_texture));
            shader.set_int(&gl, "position_texture", 1);
            gl.active_texture(TEXTURE2);
            gl.bind_texture(TEXTURE_2D, Some(self.normal_texture));
            shader.set_int(&gl, "normal_texture", 2);
            gl.active_texture(TEXTURE3);
            gl.bind_texture(TEXTURE_2D, Some(self.texcoord_texture));
            shader.set_int(&gl, "texcoord_texture", 3);

            let mut index = 0;
            for (i, mesh) in model.mesh.iter().enumerate() {
                for texture in &mesh.textures {
                    gl.active_texture(TEXTURE4 + index as u32);
                    let name = &texture.type_;
                    let number = match name.as_str() {
                        "diffuse_texture" => i,
                        "specular_texture" => i + 1,
                        "normal_texture" => i + 2,
                        "height_texture" => i + 3,
                        _ => panic!("unknown texture type"),
                    };
                    let sampler = CString::new(format!("{}{}", name, number)).unwrap();
                    shader.set_int(&gl, sampler.to_str().unwrap(), index + 4);
                    gl.bind_texture(TEXTURE_2D, Some(texture.id));
                    index += 1;
                }
            }
            gl.active_texture(TEXTURE0);
        }
    }
}
