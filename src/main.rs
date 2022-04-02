extern crate gl;
extern crate sdl2;
extern crate png;

mod sprite;

fn main() {
    use sprite::Sprite;
    use sprite::Transform; //todo move transform into another module
    use sprite::Texture;

    use std::path::Path;

    let sdl_ctx = sdl2::init().unwrap();
    let vid_sys = sdl_ctx.video().unwrap();
    let gl_attr = vid_sys.gl_attr(); {
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_flags().debug().set();
        gl_attr.set_context_version(3, 3);
    }
    let window = vid_sys.window("ClimberLD50", 512, 512)
        .position_centered()
        .opengl()
        .build().unwrap();
    let _gl_ctx = window.gl_create_context().unwrap();
    let mut event_pump = sdl_ctx.event_pump().unwrap();

    gl::load_with(|s| vid_sys.gl_get_proc_address(s) as *const _);
    Sprite::init();

    unsafe { gl::ClearColor(0.25, 0.2, 0.3, 1.0); }
    
    let hard_tex = Texture::new(Path::new("assets/thispicgoeshard1.png"));
    let evil_tex = Texture::new(Path::new("assets/thispicgoesevil1.png"));
    let mut spr = Sprite::new(hard_tex); {
        spr.transform.origin = (0.0, 1.0).into();
        spr.transform.scale = (0.5, 0.05).into();
        spr.transform.translation = (0.0, 0.5).into();
    }
    let mut square = Sprite::new(evil_tex); {
        square.transform.origin = (0.0, -1.0).into();
        square.transform.scale = (0.5, 0.4).into();
        square.transform.translation = (0.0, -0.5).into();
    }

    let mut camera = Transform::identity(); {
        camera.translation = (1.0, 0.0).into();
        //camera.scale = (8.0, 8.0).into();
        camera.rotation = std::f32::consts::PI / 4f32;
    }

    let mut frame_count = 0;

    'game: loop {
        frame_count += 1;
        for e in event_pump.poll_iter() {
            use sdl2::event::Event;

            match e {
                Event::Quit{..} => {
                    break 'game;
                }
                _ => {}
            }
        }
        camera.translation = (
            0.5 * ((frame_count as f32) / 1000f32).cos(),
            0.5 * ((frame_count as f32) / 1000f32).sin()
        ).into();
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
        Sprite::begin(camera); {
            spr.draw();
            square.draw();
        } Sprite::end();
        window.gl_swap_window();
    }
}
