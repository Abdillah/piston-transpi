use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use opengl_graphics::{GlGraphics, OpenGL};
use gl;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

type Point = [f64; 2];

fn bezier_curve_points<'a>(p1: &'a Point, p2: &Point, p3: &Point) -> Vec<Point> {
    let mut points = vec![];
    let step = 200;
    for i in 0..=step {
        let t: f64 = f64::from(i) / step as f64;
        // P(t) = P0*t^2 + P1*2*t*(1-t) + P2*(1-t)^2
        let p_x = p3[0] * t.powf(2.0) + 2.0*p2[0]*t*(1.0-t) + p1[0]*(1.0-t).powf(2.0);
        let p_y = p3[1] * t.powf(2.0) + 2.0*p2[1]*t*(1.0-t) + p1[1]*(1.0-t).powf(2.0);
        points.push([ p_x, p_y ]);
    }
    points
}

fn interp(s1: f64, s2: f64, distance: f64) -> f64 {
    let diff = s2 - s1;
    if diff == 0.0 {
        s1
    } else {
        s1 + ((s2 - s1) / (s2 - s1).abs() * distance)
    }
}

fn get_rounded_rect_points(rect: graphics::types::Rectangle, radius: f64) -> Vec<[f64; 2]> {
    let [ minx, miny, w, h ] = rect;
    let tl = [ minx, miny ];
    let tr = [ minx + w, miny ];
    let bl = [ minx, miny + h ];
    let br = [ minx + w, miny + h ];

    enum Corner {
        TopLeft,
        TopRight,
        BottomLeft,
        BottomRight,
    };

    /// Get points of corner cut by circle intersection
    ///
    /// The point returned is clockwise.
    fn get_cutcorner_points(rect: graphics::types::Rectangle, corner: Corner, radius: f64) -> [Point; 2] {
        let [ minx, miny, w, h ] = rect;
        match corner {
            Corner::TopLeft => [
                [ minx, miny + radius ],
                [ minx + radius, miny ],
            ],
            Corner::TopRight => [
                [ minx + w - radius, miny ],
                [ minx + w, miny + radius ],
            ],
            Corner::BottomRight => [
                [ minx + w, miny + h - radius ],
                [ minx + w - radius, miny + h ],
            ],
            Corner::BottomLeft => [
                [ minx + radius, miny + h ],
                [ minx, miny + h - radius ],
            ],
        }
    }

    // Resulting points:
    //
    //   .-----------------.
    //   |   ^--- tl  ^-- tr
    //   | <-- lt          | <-- rt
    //   |                 |
    let mut curved_rect_p = vec![];
    let [ tl_left, tl_top ] = get_cutcorner_points(rect, Corner::TopLeft, radius);

    curved_rect_p.push(bezier_curve_points(&tl_left, &tl, &tl_top));
    let [ tr_top, tr_right ] = get_cutcorner_points(rect, Corner::TopRight, radius);
    curved_rect_p.push(bezier_curve_points(&tr_top, &tr, &tr_right));
    let [ br_right, br_bottom ] = get_cutcorner_points(rect, Corner::BottomRight, radius);
    curved_rect_p.push(bezier_curve_points(&br_right, &br, &br_bottom));
    let [ bl_bottom, bl_left ] = get_cutcorner_points(rect, Corner::BottomLeft, radius);
    curved_rect_p.push(bezier_curve_points(&bl_bottom, &bl, &bl_left));
    let curved_rect_p: Vec<Point> = curved_rect_p.into_iter().flatten().collect();

    curved_rect_p
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    {
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 2);
    
        // Not all drivers default to 32bit color, so explicitly set it to 32bit color.
        gl_attr.set_red_size(8);
        gl_attr.set_green_size(8);
        gl_attr.set_blue_size(8);
        gl_attr.set_alpha_size(8);
        gl_attr.set_stencil_size(8);
        gl_attr.set_framebuffer_srgb_compatible(true);

        gl_attr.set_multisample_buffers(0);
        gl_attr.set_multisample_samples(0);

        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 2));
    }

    let mut window = video_subsystem
    .window("rust-sdl2 demo: Window", 800, 600)
    .opengl()
    .resizable()
    .borderless()
    // .opacity(0.0)
    .build()
    .map_err(|e| e.to_string())?;

    // Unlike the other example above, nobody created a context for your window, so you need to create one.
    let ctx = window.gl_create_context().unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    let winsize = window.size();

    let mut tick = 0;

    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string())?;

    let opengl = OpenGL::V3_2;
    let mut pgr = GlGraphics::new(opengl);

    // const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    const WHITISH: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
    const GRAY: [f32; 4] = [0.4, 0.4, 0.4, 0.9];
    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    'running: loop {
        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }

        {
            let position = window.position();
            let size = window.size();
            let title = format!("Window - pos({}x{}), size({}x{}): {}",
                                position.0,
                                position.1,
                                size.0,
                                size.1,
                                tick);
            window.set_title(&title).map_err(|e| e.to_string())?;

            tick += 1;
        }

        let (w, h): (f64, f64) = (winsize.0.into(), winsize.1.into());

        pgr.draw(viewport::Viewport {
            rect: [ 0, 0, w as i32, h as i32 ],
            draw_size: [ w as u32, h as u32 ],
            window_size: [ w.into(), h.into() ],
        }, |c, pgr| {
            use graphics::*;

            // clear([1.0; 4], pgr);

            let mut glycache = opengl_graphics::GlyphCache::new("/home/fazbdillah/.local/share/fonts/FiraCode-Regular.ttf", (), opengl_graphics::TextureSettings::new()).unwrap();
            // text(BLACK, 72, "H", &mut glycache, c.transform.trans(20.0, 72.0), pgr);

            text::Text::new_color(WHITISH, 32)
            .draw(
                "Hello opengl_graphics!",
                &mut glycache,
                &DrawState::default(),
                c.transform.trans(10.0, 100.0),
                pgr
            ).unwrap();
        });

        window.gl_swap_window();
    }

    Ok(())
}
