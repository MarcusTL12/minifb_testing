use std::time::Duration;

use minifb::{MouseButton, MouseMode, Window, WindowOptions};

fn xy_to_screencoords(x: f32, y: f32, w: usize, h: usize) -> (isize, isize) {
    let xn = ((x + 1.0) / 2.0 * w as f32) as isize;
    let yn = ((y + 1.0) / 2.0 * h as f32) as isize;
    (xn, yn)
}

fn screencoords_to_ind(x: usize, y: usize, w: usize) -> usize {
    x + y * w
}

fn xy_to_ind(x: f32, y: f32, w: usize, h: usize) -> usize {
    let (xn, yn) = xy_to_screencoords(x, y, w, h);
    screencoords_to_ind(xn as usize, yn as usize, w)
}

fn render_triangle(
    buffer: &mut [u32],
    coords: &[(f32, f32); 3],
    w: usize,
    h: usize,
) {
    let top = coords
        .iter()
        .enumerate()
        .min_by(|(_, (_, a)), (_, (_, b))| (*a).partial_cmp(b).unwrap())
        .unwrap()
        .0;
    //
    let bottom = coords
        .iter()
        .enumerate()
        .max_by(|(_, (_, a)), (_, (_, b))| (*a).partial_cmp(b).unwrap())
        .unwrap()
        .0;
    //
    let mid = 3 - top - bottom;
    //
}

fn main() {
    const W: usize = 512;
    const H: usize = 512;
    let mut window = Window::new("Test", W, H, WindowOptions::default())
        .expect("Unable to open Window");
    //
    window.limit_update_rate(Some(Duration::from_secs_f64(1.0 / 60.0)));
    //
    let mut buffer = [0; W * H];
    //
    let triangle = [(-0.5, -0.5), (0.5, 0.0), (0.0, 0.5)];
    //
    render_triangle(&mut buffer, &triangle, W, H);
    //
    while window.is_open() {
        window.update_with_buffer(&buffer, W, H).unwrap();
        //
        //
        if let Some((x, y)) = window.get_mouse_pos(MouseMode::Clamp) {
            if window.get_mouse_down(MouseButton::Left) {
                buffer[(x + W as f32 * y) as usize] = 0xff00ff;
            }
        }
    }
}
