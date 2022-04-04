extern crate gl;
extern crate sdl2;
extern crate png;
extern crate rand;

mod sprite;

fn main() {
    use sprite::Sprite;
    use sprite::Transform; //todo move transform into another module
    use sprite::Texture;
    use sprite::Vector2;

    use std::path::Path;
    use rand::prelude::*;

    let mut rng = rand::thread_rng();
    let sdl_ctx = sdl2::init().unwrap();
    let vid_sys = sdl_ctx.video().unwrap();
    let gl_attr = vid_sys.gl_attr(); {
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_flags().debug().set();
        gl_attr.set_context_version(3, 3);
    }
    let window = vid_sys.window("ClimberLD50", 1024, 512)
        .position_centered()
        .opengl()
        .build().unwrap();
    let _gl_ctx = window.gl_create_context().unwrap();
    let mut event_pump = sdl_ctx.event_pump().unwrap();

    gl::load_with(|s| vid_sys.gl_get_proc_address(s) as *const _);
    vid_sys.gl_set_swap_interval(sdl2::video::SwapInterval::VSync).unwrap();
    Sprite::init();

    unsafe { gl::ClearColor(0.25, 0.2, 0.3, 1.0); }
    
    let hard_tex = Texture::new(Path::new("assets/thispicgoeshard1.png"));
    let evil_tex = Texture::new(Path::new("assets/thispicgoesevil1.png"));

    let mut spr = Sprite::new(hard_tex); {
        spr.transform.origin = (0.0, -1.0).into();
        spr.transform.scale = (0.7, 1.0).into();
    }

    let mut camera = Transform::identity(); {
        camera.translation = (0.0, 7.0).into();
        camera.scale = (16.0, 8.0).into();
    }

    use std::time::*;

    let mut deathframe_count = 0;
    let mut time0 = Instant::now();
    let mut time_accum = Duration::ZERO;
    let mut t = Duration::ZERO;

    const DT: Duration = Duration::from_micros(13889);

    let (mut p_pos, mut p_vel): (_, Vector2) = (Vector2::zero(), (0.2, 0.5).into());
    let mut target_height = 0f32;
    let mut grounded = 0;
    let mut platforms = [(Vector2::zero(), 3f32, false); 4];

    let mut reset = || -> () {
        for (i, p) in platforms.iter_mut().enumerate() {
            p.0 = (6.0 * i as f32 - 8.0, 6.0 * i as f32).into();
            p.1 = 3f32;
            p.2 = false;
        }
        p_pos = platforms[0].0;
    };

    let mut plat_sprs = [Sprite::new(evil_tex); 4];

    let mut jump_btn = (false, false);
    
    reset();

    'game: loop {
        use sdl2::keyboard::Scancode;
        
        let time1 = Instant::now();
        let frame_dur = time1 - time0;

        let mut input_dir = 0f32;

        time0 = time1;
        time_accum += frame_dur;

        while time_accum >= DT {
            jump_btn.1 = jump_btn.0;
            for e in event_pump.poll_iter() {
                use sdl2::event::Event;

                match e {
                    Event::Quit{..} => {
                        break 'game;
                    }
                    Event::KeyDown { scancode, ..} => {
                        match scancode {
                            Some(Scancode::Space) => {
                                jump_btn.0 = true;
                            }
                            _ => {}
                        }
                    }
                    Event::KeyUp { scancode, ..} => {
                        match scancode {
                            Some(Scancode::Space) => {
                                jump_btn.0 = false;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            let key_state = event_pump.keyboard_state();

            if key_state.is_scancode_pressed(Scancode::Left) {
                input_dir -= 1f32;
            }
            if key_state.is_scancode_pressed(Scancode::Right) {
                input_dir += 1f32;
            }

            if p_pos.y - camera.translation.y < -10f32 {
                deathframe_count += 1;
                if deathframe_count > 72 {
                    for (i, p) in platforms.iter_mut().enumerate() {
                        p.0 = (6.0 * i as f32 - 8.0, 8.0 * i as f32).into();
                        p.1 = 3f32;
                        p.2 = false;
                    }
                    p_pos = platforms[0].0;
                    p_vel = Vector2::zero();
                    camera.translation.y = 7f32;
                    target_height = 7f32;
                    deathframe_count = 0;
                    continue 'game;
                } else {
                    continue;
                }
            }

            p_vel.y += -30f32 * DT.as_secs_f32();
            p_pos = p_pos + p_vel.scale(DT.as_secs_f32());

            let mut no_touchy = true;

            if p_vel.y <= 0f32 {
                for p in platforms.iter_mut() {
                    let touch_factor = if p.2 { 2f32 } else { 1f32 };

                    if 
                        (p_pos.x - p.0.x).abs() < p.1 * touch_factor &&
                        (p_pos.y - (p.0.y - 0.15)).abs() < 0.3
                    {
                        p_vel.y = 0.0;
                        p_pos.y =
                            p.0.y -
                            ((camera.translation.y - 7f32) / 32f32).min(5f32) * DT.as_secs_f32();
                        grounded = 2;
                        p.2 = true;
                        no_touchy = false;

                        if
                            p_pos.y + 7.0 > camera.translation.y &&
                            target_height <= camera.translation.y
                        {
                            target_height = p_pos.y + 7.0;
                        }
                    }
                }
            }

            if no_touchy {
                grounded = grounded.min(1);
            }

            if p_vel.x.abs() < 12f32 {
                p_vel.x += 160f32 * input_dir * DT.as_secs_f32();
            }
            if grounded > 0 && jump_btn.0 && !jump_btn.1 {
                if input_dir != 0f32 && grounded > 1 {
                    p_vel.y = 8.0;
                    p_vel.x += input_dir * 6f32;
                    grounded = 1;
                } else
                if input_dir == 0f32 || grounded == 1 {
                    p_vel.y = 24.0;
                    grounded = 0;
                }
            }
            if input_dir == 0f32 {
                p_vel.x *= 0.9f32;
            }

            for p in platforms.iter_mut() {
                if p.2 {
                    p.0.y -= ((camera.translation.y - 6f32) / 32f32).min(5f32) * DT.as_secs_f32();
                }

                if p.0.y - camera.translation.y < -8f32 {
                    p.0.x = rng.gen_range(-12f32..=12f32);
                    p.0.y += 24f32 + rng.gen_range(-0.5f32..=0.5f32);
                    p.1 = (p.1 - 0.2).max(1f32);
                    p.2 = false;
                }
            }

            t += DT;
            time_accum -= DT;
        }

        for (i, sprs) in plat_sprs.iter_mut().enumerate() {
            let touch_factor = if platforms[i].2 { 2f32 } else { 1f32 };

            sprs.transform.origin = (0.0, 1.0).into();
            sprs.transform.translation = platforms[i].0;
            sprs.transform.scale = (platforms[i].1 * touch_factor, 0.3).into();
        }

        if (camera.translation.y - target_height).abs() > 32.0 * frame_dur.as_secs_f32() {
            let dir = (target_height - camera.translation.y).signum();

            camera.translation.y += 32.0 * dir * frame_dur.as_secs_f32();
        } else {
            camera.translation.y = target_height;
        }

        spr.transform.translation = p_pos;
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
        Sprite::begin(camera); {
            spr.draw();
            for sprs in plat_sprs.iter() {
                sprs.draw();
            }
        } Sprite::end();
        window.gl_swap_window();
    }
}
