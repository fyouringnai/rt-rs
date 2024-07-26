use glow::*;
use ray_tracer::renderer::Renderer;
use ray_tracer::App;
use slint::ComponentHandle;

pub fn main() {
    let app = App::new().unwrap();

    let mut renderer = None;

    let app_weak = app.as_weak();

    if let Err(error) =
        app.window()
            .set_rendering_notifier(move |state, graphics_api| match state {
                slint::RenderingState::RenderingSetup => unsafe {
                    let context = match graphics_api {
                        slint::GraphicsAPI::NativeOpenGL { get_proc_address } => {
                            Context::from_loader_function_cstr(|s| get_proc_address(s))
                        }

                        _ => return,
                    };
                    renderer = Some(Renderer::new(context))
                },
                slint::RenderingState::BeforeRendering => {
                    if let (Some(renderer), Some(app)) = (renderer.as_mut(), app_weak.upgrade()) {
                        renderer.render(&app);
                        app.window().request_redraw();
                    }
                }
                slint::RenderingState::AfterRendering => {}
                slint::RenderingState::RenderingTeardown => {
                    drop(renderer.take());
                }
                _ => {}
            })
    {
        match error {
            slint::SetRenderingNotifierError::Unsupported => eprintln!("This example requires the use of the GL backend. Please run with the environment variable SLINT_BACKEND=GL set."),
            _ => unreachable!()
        }
        std::process::exit(1);
    }

    app.run().unwrap();
}
