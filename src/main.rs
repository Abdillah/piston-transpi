use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use opengl_graphics::{GlGraphics, OpenGL};
use gl;

type Point = [f64; 2];

fn bezier_curve_points<'a>(p1: &'a Point, p2: &Point, p3: &Point) -> Vec<Point> {
    let mut points = vec![];
    for i in 0..=100 {
        let t: f64 = f64::from(i) / 100.0;
        // P(t) = P0*t^2 + P1*2*t*(1-t) + P2*(1-t)^2
        let p_x = p1[0] * t.powf(2.0) + 2.0*p2[0]*t*(1.0-t) + p3[0]*(1.0-t).powf(2.0);
        let p_y = p1[1] * t.powf(2.0) + 2.0*p2[1]*t*(1.0-t) + p3[1]*(1.0-t).powf(2.0);
        points.push([ p_x, p_y ]);
    }
    points
}

fn interp(s1: f64, s2: f64, pct: f64) -> f64 {
    s1 + ((s2 - s1) * pct)
}

fn get_rounded_rect_points(rect: graphics::types::Rectangle, round_pct: f64) -> Vec<[f64; 2]> {
    let [ minx, miny, w, h ] = rect;
    let p1 = [ minx, miny ];
    let p2 = [ minx + w, miny ];
    let p3 = [ minx + w, miny + h ];
    let p4 = [ minx, miny + h ];

    /// Return interpolation points of a line at both ends 
    #[inline]
    fn segment_line_ends(p1: Point, p2: Point, pct: f64) -> [Point; 2] {
        [
            [ interp(p1[0], p2[0], pct), interp(p1[1], p2[1], pct) ],
            [ interp(p1[0], p2[0], 1.0 - pct), interp(p1[1], p2[1], 1.0 - pct) ],
        ]
    }

    // Resulting points:
    //
    //   .-----------------.
    //   |   ^--- tl  ^-- tr
    //   | <-- lt          | <-- rt
    //   |                 |
    let top = segment_line_ends(p1, p2, round_pct);
    let right = segment_line_ends(p2, p3, round_pct);
    let bottom = segment_line_ends(p3, p4, round_pct);
    let left = segment_line_ends(p4, p1, round_pct);

    let mut curved_rect_p = vec![];
    curved_rect_p.push(bezier_curve_points(  &left[1], &p1, &top[0]));
    curved_rect_p.push(bezier_curve_points(&bottom[1], &p4, &left[0]));
    curved_rect_p.push(bezier_curve_points( &right[1], &p3, &bottom[0]));
    curved_rect_p.push(bezier_curve_points(   &top[1], &p2, &right[0]));
    let curved_rect_p: Vec<Point> = curved_rect_p.into_iter().flatten().collect();

    curved_rect_p
}

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("A transparent window!")
        .with_decorations(false)
        .with_transparent(true);

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
                use graphics::*;

                // const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
                const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
                const GRAY: [f32; 4] = [0.4, 0.4, 0.4, 0.9];

                let square = rectangle::square(0.0, 0.0, 100.0);
                let (x, y) = (20.0 / 2.0, 20.0 / 2.0);
        
                pgr.draw(viewport::Viewport {
                    rect: [ 20, 20, 120, 120 ],
                    draw_size: [ 100, 100 ],
                    window_size: [ 100.0, 100.0 ],
                }, |c, pgr| {
                    // Clear the screen.
                    // clear(GREEN, pgr);
        
                    // let transform = c
                    //     .transform
                    //     .trans(x, y)
                    //     // .rot_rad(20.0)
                    //     .trans(-25.0, -25.0);

                    // let rect_p: Vec<[f64; 2]> = vec![
                    //     [   0.0,   0.0 ],
                    //     [  50.0,   0.0 ],
                    //     [ 100.0,   0.0 ],
                    //     [ 100.0,  50.0 ],
                    //     [ 100.0, 100.0 ],
                    //     [  50.0, 100.0 ],
                    //     [   0.0, 100.0 ],
                    //     [   0.0,  50.0 ],
                    // ];
                    // let mut curved_rect_p = vec![];
                    // curved_rect_p.push(bezier_curve_points(&rect_p[7], &rect_p[0], &rect_p[1]));
                    // curved_rect_p.push(bezier_curve_points(&rect_p[1], &rect_p[2], &rect_p[3]));
                    // curved_rect_p.push(bezier_curve_points(&rect_p[3], &rect_p[4], &rect_p[5]));
                    // curved_rect_p.push(bezier_curve_points(&rect_p[5], &rect_p[6], &rect_p[7]));
                    // let curved_rect_p: Vec<[f64; 2]> = curved_rect_p.into_iter().flatten().collect();
                    
                    let rect = graphics::rectangle::rectangle_by_corners(0.0, 0.0, 100.0, 100.0);
                    let curved_rect_p = get_rounded_rect_points(rect, 0.1);

                    // println!("Curved rect: {:?}", curved_rect_p.iter().map(|p| p[0]).collect::<Vec<f64>>());

                    polygon(GRAY, &curved_rect_p.as_slice(), c.transform, pgr);

                    // Draw a box rotating around the middle of the screen.
                    // rectangle(RED, square, c.transform, pgr);
                });

                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
