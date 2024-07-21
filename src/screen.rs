use std::mem::size_of;

use crate::fbo::ScreenFBO;
use crate::shader::Shader;
use bytemuck::cast_slice;
use glow::{
    Context, HasContext, VertexArray, ARRAY_BUFFER, FLOAT, NO_ERROR, STATIC_DRAW, TRIANGLES,
};

pub struct Screen {
    pub shader: Shader,
    vao: VertexArray,
}

impl Screen {
    pub fn new(gl: &Context) -> Self {
        let vertices: [f32; 30] = [
            -1.0, 1.0, 0.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            1.0, 0.0, 0.0, 1.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0,
        ];
        let vao = unsafe { gl.create_vertex_array().unwrap() };
        let vbo = unsafe { gl.create_buffer().unwrap() };
        unsafe {
            assert_eq!(gl.get_error(), NO_ERROR);
        }

        let shader = Shader::new(gl, "shaders/screen.vert", "shaders/screen.frag");
        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, cast_slice(&vertices), STATIC_DRAW);
            gl.vertex_attrib_pointer_f32(0, 3, FLOAT, false, (5 * size_of::<f32>()) as i32, 0);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                FLOAT,
                false,
                (5 * size_of::<f32>()) as i32,
                (3 * size_of::<f32>()) as i32,
            );
            gl.enable_vertex_attrib_array(0);
            gl.enable_vertex_attrib_array(1);
            gl.bind_buffer(ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
        }
        Self { shader, vao }
    }

    pub fn draw(&self, gl: &Context) {
        unsafe {
            self.shader.use_program(gl);
            self.shader.set_int(gl, "screenTexture", 0);
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(TRIANGLES, 0, 6);
        }
    }

    pub fn draw_shader(&self, gl: &Context, shader: &Shader) {
        unsafe {
            shader.use_program(gl);
            shader.set_int(gl, "historyTexture", 0);
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(TRIANGLES, 0, 6);
        }
    }
}

pub struct ScreenBuffer {
    fbo: [ScreenFBO; 2],
    width: i32,
    height: i32,
}

impl ScreenBuffer {
    pub fn new(gl: &Context, width: i32, height: i32) -> Self {
        let fbo = [
            ScreenFBO::new(gl, width, height),
            ScreenFBO::new(gl, width, height),
        ];
        let width = width;
        let height = height;

        Self { fbo, width, height }
    }

    pub fn set_current_buffer(&mut self, gl: &Context, render_loop: i32) {
        let last_buffer = render_loop % 2;
        if last_buffer == 0 {
            self.fbo[1].bind(gl);
            self.fbo[0].bind_texture(gl);
        } else {
            self.fbo[0].bind(gl);
            self.fbo[1].bind_texture(gl);
        }
    }

    pub fn set_current_texture(&mut self, gl: &Context, render_loop: i32) {
        let last_buffer = render_loop % 2;
        if last_buffer == 0 {
            self.fbo[1].bind_texture(gl);
        } else {
            self.fbo[0].bind_texture(gl);
        }
    }

    pub fn resize(&mut self, gl: &Context, width: i32, height: i32) {
        if self.width != width || self.height != height {
            self.fbo[0].delete(gl);
            self.fbo[1].delete(gl);

            self.fbo = [
                ScreenFBO::new(gl, width, height),
                ScreenFBO::new(gl, width, height),
            ];
        }
    }

    pub fn delete(&self, gl: &Context) {
        self.fbo[0].delete(gl);
        self.fbo[1].delete(gl);
    }
}
