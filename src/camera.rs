use crate::shader::Shader;
use crate::App;
use cgmath::*;
use glow::Context;
use slint::PhysicalSize;

pub struct Camera {
    pub position: Point3<f32>,
    pub front: Vector3<f32>,
    pub right: Vector3<f32>,
    pub up: Vector3<f32>,
    pub world_up: Vector3<f32>,
    pub left_bottom: Vector3<f32>,

    pub movement_speed: f32,
    pub fov: f32,
    pub wheel_sensitivity: f32,

    pub pitch: f32,
    pub yaw: f32,
    pub mouse_sensitivity: f32,

    pub first_mouse: bool,
    pub last_x: f32,
    pub last_y: f32,

    pub width: i32,
    pub height: i32,

    pub render_loop: i32,
}

impl Default for Camera {
    fn default() -> Self {
        let mut camera = Self {
            position: Point3::new(0.0, 0.0, 0.0),
            front: Vector3::new(0.0, 0.0, -1.0),
            up: Vector3::zero(),
            right: Vector3::zero(),
            world_up: Vector3::unit_y(),
            left_bottom: Vector3::zero(),

            movement_speed: 2.5,
            fov: 60.0,
            wheel_sensitivity: 0.1,

            pitch: 0.0,
            yaw: -90.0,
            mouse_sensitivity: 0.1,

            first_mouse: true,
            last_x: 0.0,
            last_y: 0.0,

            width: 0,
            height: 0,

            render_loop: 0,
        };
        camera.update_camera_vectors();
        camera
    }
}

impl Camera {
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        return Matrix4::look_at_rh(self.position, self.position + self.front, self.up);
    }

    pub fn process_keyboard(&mut self, app: &App, delta_time: f32) {
        let camera_speed = self.movement_speed * delta_time;
        let front = vec3(self.front.x, 0.0, self.front.z).normalize();
        let right = vec3(self.right.x, 0.0, self.right.z).normalize();
        let up = vec3(0.0, self.up.y, 0.0).normalize();
        if app.get_movement_up() {
            self.position += front * camera_speed;
            self.update_camera_vectors();
        }
        if app.get_movement_down() {
            self.position -= front * camera_speed;
            self.update_camera_vectors();
        }
        if app.get_movement_left() {
            self.position -= right * camera_speed;
            self.update_camera_vectors();
        }
        if app.get_movement_right() {
            self.position += right * camera_speed;
            self.update_camera_vectors();
        }
        if app.get_movement_float() {
            self.position += up * camera_speed;
            self.update_camera_vectors();
        }
        if app.get_movement_drown() {
            self.position -= up * camera_speed;
            self.update_camera_vectors();
        }
    }

    pub fn process_mouse_movement(&mut self, app: &App) {
        if app.get_mouse_pressed() == true {
            let current_x = app.get_mouse_position_x() as f32;
            let current_y = app.get_mouse_position_y() as f32;
            if self.first_mouse == true {
                self.last_x = current_x;
                self.last_y = current_y;
                self.first_mouse = false;
            }
            let mut x_offset = current_x - self.last_x;
            let mut y_offset = current_y - self.last_y;

            self.last_x = current_x;
            self.last_y = current_y;

            x_offset *= self.mouse_sensitivity;
            y_offset *= self.mouse_sensitivity;

            self.yaw += x_offset;
            self.pitch -= y_offset;

            if self.pitch >= 90.0 {
                self.pitch = 90.0;
            }
            if self.pitch <= -90.0 {
                self.pitch = -90.0;
            }

            self.update_camera_vectors();
        } else {
            self.first_mouse = true;
        }
    }

    pub fn process_mouse_wheel(&mut self, app: &App) {
        let offset = self.wheel_sensitivity * app.get_mouse_wheel_offset() as f32;
        self.fov += offset;

        if self.fov < 1.0 {
            self.fov = 1.0;
        }
        if self.fov > 90.0 {
            self.fov = 90.0;
        }
        self.update_camera_vectors();
    }

    pub fn update_ratio(&mut self, width: i32, height: i32) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.update_camera_vectors();
        }
    }

    pub fn update_loop(&mut self) {
        self.render_loop = self.render_loop + 1;
    }

    pub fn use_camera(&self, gl: &Context, shader: &Shader, size: &PhysicalSize) {
        shader.set_int(gl, "screenWidth", size.width as i32);
        shader.set_int(gl, "screenHeight", size.height as i32);
        shader.set_vector3(gl, "camera.camPos", &self.position.to_vec());
        shader.set_vector3(gl, "camera.front", &self.front);
        shader.set_vector3(gl, "camera.right", &self.right);
        shader.set_vector3(gl, "camera.up", &self.up);
        shader.set_vector3(gl, "camera.leftbottom", &self.left_bottom);

        shader.set_float(&gl, "camera.halfH", (self.fov / 2.0).to_radians().tan());
        shader.set_float(
            gl,
            "camera.halfW",
            (size.width as f32 / size.height as f32) * (self.fov / 2.0).to_radians().tan(),
        );
        shader.set_int(gl, "camera.LoopNum", self.render_loop);
    }

    fn update_camera_vectors(&mut self) {
        let front = vec3(
            self.yaw.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin(),
        );

        self.front = front.normalize();
        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
        self.left_bottom = self.front
            - self.right
                * (self.width as f32 / self.height as f32)
                * (self.fov / 2.0).to_radians().tan()
            - self.up * (self.fov / 2.0).to_radians().tan();
    }
}
