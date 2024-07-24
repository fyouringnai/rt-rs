use std::time::Instant;

use cgmath::point3;
use glow::{Context, HasContext, COLOR_BUFFER_BIT, FRAMEBUFFER};
use slint::ComponentHandle;

use crate::bvh::BVHTree;
use crate::camera::Camera;
use crate::model::Model;
use crate::object::Object;
use crate::screen::{Screen, ScreenBuffer};
use crate::shader::Shader;
use crate::utils::MATERIAL::*;
use crate::App;

pub struct Renderer {
    gl: Context,
    camera: Camera,
    screen: Screen,
    shader: Shader,
    model: Model,
    bvh_tree: BVHTree,
    primitives: Vec<Object>,
    screen_buffer: ScreenBuffer,
    frame_time: f32,
    frame_count: i32,
    last_frame: Instant,
}

impl Renderer {
    pub fn new(gl: Context) -> Renderer {
        let screen = Screen::new(&gl);
        let camera = Camera {
            position: point3(0.0, 1.0, 3.0),
            ..Camera::default()
        };
        let shader = Shader::new(
            &gl,
            "shaders/path_tracing.vert",
            "shaders/path_tracing.frag",
        );
        let screen_buffer = ScreenBuffer::new(&gl, 1600, 1200);
        let model = unsafe { Model::new(&gl, "models/furina_w/furina.obj") };
        let mut bvh_tree = BVHTree::new(&gl);
        let mut primitives = Vec::new();
        let sphere = Object::new_sphere([-2.0, 1.0, 0.0], 0.3, [0.8, 0.8, 0.8], METAL);
        let floor_vert = vec![
            [-10.0, 0.0, -10.0],
            [-10.0, 0.0, 10.0],
            [10.0, 0.0, 10.0],
            [10.0, 0.0, -10.0],
            [0.0, 1.0, 0.0],
        ];
        let floor = Object::new_rectangle(floor_vert, [0.8, 0.8, 0.8], METAL);

        primitives.push(sphere);
        primitives.push(floor);
        model.get_primitives(&mut primitives, DIFFUSE);
        bvh_tree.build(&primitives);
        bvh_tree.set_texture(&gl);
        let renderer = Renderer {
            gl,
            camera,
            screen,
            shader,
            model,
            bvh_tree,
            primitives,
            screen_buffer,
            frame_time: 0.0,
            frame_count: 0,
            last_frame: Instant::now(),
        };
        renderer
    }
    pub fn render(&mut self, app: &App) {
        let size = app.window().size();
        let current_frame = Instant::now();
        let delta_time = current_frame.duration_since(self.last_frame).as_secs_f32();
        if self.frame_time > 1.0 {
            app.set_fps(format!("{:.2} fps", self.frame_count as f32 / self.frame_time).into());
            self.frame_time = 0.0;
            self.frame_count = 0;
        } else {
            self.frame_time += delta_time;
            self.frame_count += 1;
        }
        self.last_frame = current_frame;
        self.camera
            .update_ratio(size.width as i32, size.height as i32);
        self.camera.process_keyboard(app, delta_time);
        self.camera.process_mouse_movement(app);
        self.camera.process_mouse_wheel(app);
        self.screen_buffer
            .resize(&self.gl, size.width as i32, size.height as i32);
        self.camera.render_loop = 0;
        unsafe {
            for _i in 0..app.get_sample_counts() as i32 {
                self.camera.update_loop();
                self.screen_buffer
                    .set_current_buffer(&self.gl, self.camera.render_loop);

                self.shader.use_program(&self.gl);
                self.bvh_tree.use_texture(&self.gl, &self.shader);
                self.model.use_textures(&self.gl, &self.shader);
                self.camera.use_camera(&self.gl, &self.shader, &size);

                self.shader
                    .set_int(&self.gl, "depths", app.get_depths() as i32);

                self.screen.draw_shader(&self.gl, &self.shader);

                self.screen_buffer
                    .set_current_buffer(&self.gl, self.camera.render_loop + 1);
                self.screen.draw(&self.gl);
            }
            self.gl.bind_framebuffer(FRAMEBUFFER, None);
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(COLOR_BUFFER_BIT);
            self.screen_buffer
                .set_current_texture(&self.gl, self.camera.render_loop + 1);
            self.screen.draw(&self.gl);
        }
    }
}
