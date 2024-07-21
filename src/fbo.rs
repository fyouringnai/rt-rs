use glow::*;

pub struct ScreenFBO {
    pub fbo: Framebuffer,
    texture: Texture,
}

impl ScreenFBO {
    pub fn new(gl: &Context, width: i32, height: i32) -> Self {
        let fbo = unsafe { gl.create_framebuffer().unwrap() };
        let texture = unsafe { gl.create_texture().unwrap() };
        unsafe {
            assert_eq!(gl.get_error(), NO_ERROR);
        }

        unsafe {
            gl.bind_framebuffer(FRAMEBUFFER, Some(fbo));
            gl.bind_texture(TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                TEXTURE_2D, 0, RGB as i32, width, height, 0, RGB, FLOAT, None,
            );
            assert_eq!(gl.get_error(), NO_ERROR);

            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as i32);

            gl.framebuffer_texture_2d(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, Some(texture), 0);

            if gl.check_framebuffer_status(FRAMEBUFFER) != FRAMEBUFFER_COMPLETE {
                println!("Framebuffer not complete!");
            }

            let screen_fbo = ScreenFBO { fbo, texture };

            screen_fbo.unbind(gl);

            screen_fbo
        }
    }

    pub fn bind(&self, gl: &Context) {
        unsafe {
            gl.bind_framebuffer(FRAMEBUFFER, Some(self.fbo));
            gl.disable(DEPTH_TEST);
        }
    }

    pub fn unbind(&self, gl: &Context) {
        unsafe {
            gl.bind_framebuffer(FRAMEBUFFER, None);
        }
    }

    pub fn bind_texture(&self, gl: &Context) {
        unsafe {
            gl.active_texture(TEXTURE0);
            gl.bind_texture(TEXTURE_2D, Some(self.texture))
        }
    }

    pub fn delete(&self, gl: &Context) {
        unsafe {
            self.unbind(gl);
            gl.delete_framebuffer(self.fbo);
            gl.delete_texture(self.texture);
        }
    }
}
