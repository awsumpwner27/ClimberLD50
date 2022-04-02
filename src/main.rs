extern crate gl;
extern crate sdl2;

mod sprite;

fn main() {
    use sprite::Sprite;

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

    unsafe { gl::ClearColor(0.7, 0.4, 0.5, 1.0); }

    let mut spr = Sprite::new(); {
        spr.transform.scale = (0.1, 0.3).into();
        spr.transform.rotation = 3.1415926535f32 / 4f32;
        spr.transform.translation = (0.1, -0.2).into();
    }

    'game: loop {
        for e in event_pump.poll_iter() {
            use sdl2::event::Event;

            match e {
                Event::Quit{..} => {
                    break 'game;
                }
                _ => {}
            }
        }
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
        Sprite::begin(); {
            spr.draw();
            Sprite::end();
        }
        window.gl_swap_window();
    }
}
