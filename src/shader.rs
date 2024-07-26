use cgmath::{Matrix4, Vector3};
use glow::*;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
pub struct Shader {
    id: NativeProgram,
}

impl Shader {
    pub fn new(gl: &Context, vertex_path: &str, fragment_path: &str) -> Shader {
        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let shader = Shader { id: program };

            let mut v_shader_file = File::open(vertex_path)
                .unwrap_or_else(|_| panic!("Failed to open {}", vertex_path));
            let mut f_shader_file = File::open(fragment_path)
                .unwrap_or_else(|_| panic!("Failed to open {}", fragment_path));

            let mut vertex_code = String::new();
            let mut fragment_code = String::new();

            v_shader_file
                .read_to_string(&mut vertex_code)
                .expect("Failed to read vertex shader");
            f_shader_file
                .read_to_string(&mut fragment_code)
                .expect("Failed to read fragment shader");

            let v_shader_code = CString::new(vertex_code.as_bytes()).unwrap();
            let f_shader_code = CString::new(fragment_code.as_bytes()).unwrap();

            let vertex_shader = gl.create_shader(VERTEX_SHADER).unwrap();
            gl.shader_source(vertex_shader, v_shader_code.to_str().unwrap());
            gl.compile_shader(vertex_shader);
            if !gl.get_shader_compile_status(vertex_shader) {
                panic!(
                    "Failed to compile vertex shader: {}",
                    gl.get_shader_info_log(vertex_shader)
                );
            }
            gl.attach_shader(program, vertex_shader);

            let fragment_shader = gl.create_shader(FRAGMENT_SHADER).unwrap();
            gl.shader_source(fragment_shader, f_shader_code.to_str().unwrap());
            gl.compile_shader(fragment_shader);
            if !gl.get_shader_compile_status(fragment_shader) {
                panic!(
                    "Failed to compile fragment shader: {}",
                    gl.get_shader_info_log(fragment_shader)
                );
            }
            gl.attach_shader(program, fragment_shader);

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!(
                    "Failed to link program: {}",
                    gl.get_program_info_log(program)
                );
            }

            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);

            shader
        }
    }

    pub fn use_program(&self, gl: &Context) {
        unsafe {
            gl.use_program(Some(self.id));
        }
    }

    pub fn set_int(&self, gl: &Context, name: &str, value: i32) {
        unsafe {
            let location = gl.get_uniform_location(self.id, name);
            if let Some(location) = location {
                gl.uniform_1_i32(Some(&location), value);
            }
        }
    }

    pub fn set_bool(&self, gl: &Context, name: &str, value: bool) {
        unsafe {
            let location = gl.get_uniform_location(self.id, name);
            if let Some(location) = location {
                gl.uniform_1_i32(Some(&location), value as i32);
            }
        }
    }

    pub fn set_float(&self, gl: &Context, name: &str, value: f32) {
        unsafe {
            let location = gl.get_uniform_location(self.id, name);
            if let Some(location) = location {
                gl.uniform_1_f32(Some(&location), value);
            }
        }
    }

    pub fn set_vector3(&self, gl: &Context, name: &str, value: &Vector3<f32>) {
        unsafe {
            let location = gl.get_uniform_location(self.id, name);
            if let Some(location) = location {
                gl.uniform_3_f32(Some(&location), value.x, value.y, value.z);
            }
        }
    }

    pub fn set_vec3(&self, gl: &Context, name: &str, x: f32, y: f32, z: f32) {
        unsafe {
            let location = gl.get_uniform_location(self.id, name);
            if let Some(location) = location {
                gl.uniform_3_f32(Some(&location), x, y, z);
            }
        }
    }

    pub fn set_mat4(&self, gl: &Context, name: &str, value: &Matrix4<f32>) {
        unsafe {
            let value_array: [f32; 16] = *value.as_ref();
            let location = gl.get_uniform_location(self.id, name);
            if let Some(location) = location {
                gl.uniform_matrix_4_f32_slice(Some(&location), false, &value_array);
            }
        }
    }

    pub fn delete(&self, gl: &Context) {
        unsafe {
            gl.delete_program(self.id);
        }
    }
}
