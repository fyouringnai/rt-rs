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
    depths: f32,
    width: i32,
    height: i32,
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
        let floor_vert = vec![
            [-1.0, 0.0, -1.0],
            [-1.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 0.0, -1.0],
            [0.0, 1.0, 0.0],
        ];
        let floor = Object::new_rectangle(floor_vert, [0.73, 0.73, 0.73], DIFFUSE);
        let right_wall_vert = vec![
            [1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            [1.0, 2.0, 1.0],
            [1.0, 2.0, -1.0],
            [-1.0, 0.0, 0.0],
        ];
        let right_wall = Object::new_rectangle(right_wall_vert, [0.65, 0.05, 0.05], DIFFUSE);
        let left_wall_vert = vec![
            [-1.0, 0.0, -1.0],
            [-1.0, 0.0, 1.0],
            [-1.0, 2.0, 1.0],
            [-1.0, 2.0, -1.0],
            [1.0, 0.0, 0.0],
        ];
        let left_wall = Object::new_rectangle(left_wall_vert, [0.12, 0.45, 0.15], DIFFUSE);
        let ceiling_vert = vec![
            [-1.0, 2.0, -1.0],
            [-1.0, 2.0, 1.0],
            [1.0, 2.0, 1.0],
            [1.0, 2.0, -1.0],
            [0.0, -1.0, 0.0],
        ];
        let ceiling = Object::new_rectangle(ceiling_vert, [0.73, 0.73, 0.73], DIFFUSE);
        let back_wall_vert = vec![
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 2.0, -1.0],
            [-1.0, 2.0, -1.0],
            [0.0, 0.0, 1.0],
        ];
        let back_wall = Object::new_rectangle(back_wall_vert, [1.0, 1.0, 1.0], DIFFUSE);
        let ceiling_light_vert = vec![
            [-0.23,1.99, -0.23],
            [-0.23, 1.99, 0.23],
            [0.23, 1.99, 0.23],
            [0.23, 1.99, -0.23],
            [0.0, -1.0, 0.0],
        ];
        let ceiling_light =
            Object::new_rectangle(ceiling_light_vert, [15.0, 15.0, 15.0], DIFFUSE_LIGHT);
        primitives.push(floor);
        primitives.push(right_wall);
        primitives.push(left_wall);
        primitives.push(ceiling);
        primitives.push(back_wall);
        primitives.push(ceiling_light);
        //model.get_primitives(&mut primitives, DIFFUSE);
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
            depths: 0.0,
            width: 1600,
            height: 1200,
        };
        renderer
    }
    pub fn render(&mut self, app: &App) {
        self.initialize(app);
        if app.get_real_time() {
            self.real_time_render(app);
        } else {
            self.static_render(app);
        }
    }

    fn real_time_render(&mut self, app: &App) {
        self.camera.render_loop = 0;
        self.depths = app.get_depths();
        for _i in 0..app.get_sample_counts() as i32 {
            self.renderer_core(app);
        }
        self.renderer_draw();
        self.camera.render_loop = 0;
    }

    fn static_render(&mut self, app: &App) {
        if self.depths != app.get_depths() {
            self.camera.render_loop = 0;
            self.depths = app.get_depths();
        }
        self.renderer_core(app);
        self.renderer_draw();
    }

    fn initialize(&mut self, app: &App) {
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
        self.camera.process_keyboard(app, delta_time);
        self.camera.process_mouse_movement(app);
        self.camera.process_mouse_wheel(app);
        self.camera
            .update_ratio(size.width as i32, size.height as i32);
        if self.width != size.width as i32 || self.height != size.height as i32 {
            self.width = size.width as i32;
            self.height = size.height as i32;
            self.screen_buffer
                .resize(&self.gl, size.width as i32, size.height as i32);
            self.camera.render_loop = 0;
        }
    }

    fn renderer_core(&mut self, app: &App) {
        let size = app.window().size();
        self.camera.update_loop();
        self.screen_buffer
            .set_current_buffer(&self.gl, self.camera.render_loop);

        self.shader.use_program(&self.gl);
        self.bvh_tree.use_texture(&self.gl, &self.shader);
        self.model.use_textures(&self.gl, &self.shader);
        self.camera.use_camera(&self.gl, &self.shader, &size);

        self.shader.set_int(&self.gl, "depths", self.depths as i32);

        self.screen.draw_shader(&self.gl, &self.shader);

        self.screen_buffer
            .set_current_buffer(&self.gl, self.camera.render_loop + 1);
        self.screen.draw(&self.gl);
    }

    fn renderer_draw(&mut self) {
        unsafe {
            self.gl.bind_framebuffer(FRAMEBUFFER, None);
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(COLOR_BUFFER_BIT);
            self.screen_buffer
                .set_current_texture(&self.gl, self.camera.render_loop + 1);
            self.screen.draw(&self.gl);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.model.delete(&self.gl);
        self.bvh_tree.delete_texture(&self.gl);
        self.screen_buffer.delete(&self.gl);
        self.shader.delete(&self.gl);
    }
}
