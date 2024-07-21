use crate::bvh::BVHTree;
use crate::camera::Camera;
use crate::model::Model;
use crate::object::Object;
use crate::screen::{Screen, ScreenBuffer};
use crate::shader::Shader;
use crate::utils::random_float;
use crate::App;
use cgmath::{point3, vec3, EuclideanSpace};
use glow::{Context, HasContext, COLOR_BUFFER_BIT, FRAMEBUFFER};
use slint::ComponentHandle;
use std::time::Instant;

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
        model.get_primitives(&mut primitives);
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
                self.camera.use_camera(&self.gl, &self.shader, &size);

                self.shader.set_int(&self.gl, "historyTexture", 0);
                self.shader
                    .set_float(&self.gl, "randOrigin", 674764.0 * (1.0 + random_float()));
                self.shader
                    .set_int(&self.gl, "depths", app.get_depths() as i32);

                self.shader.set_vector3(
                    &self.gl,
                    "sphere[0].center",
                    &point3(0.0, 0.0, -1.0).to_vec(),
                );
                self.shader.set_float(&self.gl, "sphere[0].radius", 0.5);
                self.shader.set_int(&self.gl, "sphere[0].material", 0);
                self.shader
                    .set_vector3(&self.gl, "sphere[0].albedo", &vec3(0.8, 0.7, 0.2));
                self.shader.set_vector3(
                    &self.gl,
                    "sphere[1].center",
                    &point3(1.0, 0.0, -1.0).to_vec(),
                );
                self.shader.set_float(&self.gl, "sphere[1].radius", 0.5);
                self.shader.set_int(&self.gl, "sphere[1].material", 1);
                self.shader
                    .set_vector3(&self.gl, "sphere[1].albedo", &vec3(0.2, 0.7, 0.6));
                self.shader.set_vector3(
                    &self.gl,
                    "sphere[2].center",
                    &point3(-1.0, 0.0, -1.0).to_vec(),
                );
                self.shader.set_float(&self.gl, "sphere[2].radius", 0.5);
                self.shader.set_int(&self.gl, "sphere[2].material", 1);
                self.shader
                    .set_vector3(&self.gl, "sphere[2].albedo", &vec3(0.1, 0.3, 0.7));
                self.shader.set_vector3(
                    &self.gl,
                    "sphere[3].center",
                    &point3(0.0, 0.0, 0.0).to_vec(),
                );
                self.shader.set_float(&self.gl, "sphere[3].radius", 0.5);
                self.shader.set_int(&self.gl, "sphere[3].material", 0);
                self.shader
                    .set_vector3(&self.gl, "sphere[3].albedo", &vec3(0.9, 0.9, 0.9));

                self.shader.set_vector3(
                    &self.gl,
                    "triangle[0].v[0]",
                    &point3(2.0, -0.5, 2.0).to_vec(),
                );
                self.shader.set_vector3(
                    &self.gl,
                    "triangle[0].v[1]",
                    &point3(-2.0, -0.5, -2.0).to_vec(),
                );
                self.shader.set_vector3(
                    &self.gl,
                    "triangle[0].v[2]",
                    &point3(-2.0, -0.5, 2.0).to_vec(),
                );

                self.shader.set_vector3(
                    &self.gl,
                    "triangle[1].v[0]",
                    &point3(2.0, -0.5, 2.0).to_vec(),
                );
                self.shader.set_vector3(
                    &self.gl,
                    "triangle[1].v[1]",
                    &point3(2.0, -0.5, -2.0).to_vec(),
                );
                self.shader.set_vector3(
                    &self.gl,
                    "triangle[1].v[2]",
                    &point3(-2.0, -0.5, -2.0).to_vec(),
                );

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
