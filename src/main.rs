use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use opengl_graphics::{GlGraphics, OpenGL};
use gl;

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
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("A transparent window!")
        .with_decorations(false)
        .with_transparent(true)
        .with_inner_size(glutin::dpi::PhysicalSize::new(500, 300))
    ;

    let windowed_context =
        ContextBuilder::new()
        .with_gl(glutin::GlRequest::GlThenGles {
            opengl_version: (3 as u8, 2 as u8),
            opengles_version: (3 as u8, 2 as u8),
        })
        .build_windowed(wb, &el).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);

    println!(
        "Pixel format of the window's GL context: {:?}",
        windowed_context.get_pixel_format()
    );

    let opengl = OpenGL::V3_2;
    let mut pgr = GlGraphics::new(opengl);

    el.run(move |event, _, control_flow| {
        // println!("{:?}", event);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(physical_size)
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                // const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
                const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
                const WHITISH: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
                const GRAY: [f32; 4] = [0.4, 0.4, 0.4, 0.9];
                const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
                const GREEN: [f32; 4] = [0.0, 0.5, 0.0, 1.0];

                let winsize = windowed_context.window().inner_size().clone();

                let w: f64 = winsize.width.into();
                let h: f64 = winsize.height.into();

                // let rust_logo = opengl_graphics::Texture::from_path(
                //     &std::path::Path::new("./assets/rust.png"),
                //     &opengl_graphics::TextureSettings::new()
                // ).unwrap();

                let fontsize = 30;

                let mut glycache = opengl_graphics::GlyphCache::new("/home/fazbdillah/.local/share/fonts/FiraCode-Regular.ttf", (), opengl_graphics::TextureSettings::new()).unwrap();
                glycache.preload_printable_ascii(fontsize);

                pgr.draw(viewport::Viewport {
                    rect: [ 0, 0, w as i32, h as i32 ],
                    draw_size: [ w as u32, h as u32 ],
                    window_size: [ w.into(), h.into() ],
                }, |c, pgr| {
                    use graphics::*;
                    use graphics::character::CharacterCache;

                    // Clear the screen.
                    // clear(WHITISH, pgr);
                    // clear([1.0; 4], pgr);
                    
                    let rect = graphics::rectangle::rectangle_by_corners(0.0, 0.0, w, 50.0);
                    let curved_rect_p = get_rounded_rect_points(rect, 10.0);
                    // println!("Curved rect: {:?}", curved_rect_p.iter().map(|p| p[0]).collect::<Vec<f64>>());

                    polygon(GRAY, &curved_rect_p.as_slice(), c.transform, pgr);

                    // text(BLACK, 72, "H", &mut glycache, c.transform.trans(20.0, 72.0), pgr);

                    unsafe { gl::Enable(gl::TEXTURE_2D); };
                    // image(&rust_logo, c.transform.trans(5.0, 50.0), pgr);

                    text::Text::new_color(WHITE, fontsize)
                    .draw(
                        "Hello World!",
                        &mut glycache,
                        &DrawState::default(),
                        c.transform.trans(50.0, 40.0),
                        pgr
                    ).unwrap();
                    unsafe { gl::Disable(gl::TEXTURE_2D); };

                    // Draw a box rotating around the middle of the screen.
                    // rectangle(RED, square, c.transform, pgr);
                });

                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
