// use std::time::Duration;

use itertools::Itertools;

use num::clamp;

use minifb::{MouseButton, MouseMode, Scale, ScaleMode, Window, WindowOptions};

fn xy_to_screencoords(x: f32, y: f32, w: usize, h: usize) -> (isize, isize) {
    let xn = ((x + 1.0) / 2.0 * w as f32) as isize;
    let yn = ((y + 1.0) / 2.0 * h as f32) as isize;
    (xn, yn)
}

fn screencoords_to_ind(x: usize, y: usize, w: usize) -> usize {
    x + y * w
}

fn screencoords_to_xy(x: f32, y: f32, w: usize, h: usize) -> (f32, f32) {
    let xn = (x / w as f32) * 2.0 - 1.0;
    let yn = (y / h as f32) * 2.0 - 1.0;
    (xn, yn)
}

// fn rgb_to_col(r: u8, g: u8, b: u8) -> u32 {
//     ((r as u32) << 16) | ((g as u32) << 8) | b as u32
// }

fn argb_to_col(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

fn texcoord_to_col(texture: &[u32], w: usize, h: usize, x: f32, y: f32) -> u32 {
    let xn = (x * w as f32) as isize;
    let yn = (y * h as f32) as isize;
    let xn = clamp(xn, 0, (w - 1) as isize) as usize;
    let yn = clamp(yn, 0, (h - 1) as isize) as usize;
    texture[screencoords_to_ind(xn, yn, w)]
}

fn render_triangle(
    buffer: &mut [u32],
    coords: &[(f32, f32)],
    texcoords: &[(f32, f32)],
    w: usize,
    h: usize,
    texture: &[u32],
    wt: usize,
    ht: usize,
) {
    let coords = {
        let mut ncoords = [(0.0f32, 0.0f32); 3];
        for ((xn, yn), (x, y)) in ncoords.iter_mut().zip(coords.iter()) {
            let (x, y) = xy_to_screencoords(*x, *y, w, h);
            *xn = x as f32;
            *yn = y as f32;
        }
        ncoords
    };
    //
    let top = coords
        .iter()
        .enumerate()
        .min_by(|(_, (_, a)), (_, (_, b))| (*a).partial_cmp(b).unwrap())
        .unwrap()
        .0;
    //
    let bot = coords
        .iter()
        .enumerate()
        .max_by(|(_, (_, a)), (_, (_, b))| (*a).partial_cmp(b).unwrap())
        .unwrap()
        .0;
    //
    let mid = 3 - top - bot;
    //
    let (xt, yt) = {
        let u1 = coords[bot].0 - coords[top].0;
        let u2 = coords[bot].1 - coords[top].1;
        let v1 = coords[mid].0 - coords[top].0;
        let v2 = coords[mid].1 - coords[top].1;
        //
        let ut1 = texcoords[bot].0 - texcoords[top].0;
        let ut2 = texcoords[bot].1 - texcoords[top].1;
        let vt1 = texcoords[mid].0 - texcoords[top].0;
        let vt2 = texcoords[mid].1 - texcoords[top].1;
        //
        let det = u1 * v2 - u2 * v1;
        if det == 0.0 {
            return;
        }
        let xxt = (v2 * ut1 - u2 * vt1) / det;
        let yxt = (u1 * vt1 - v1 * ut1) / det;
        let xyt = (v2 * ut2 - u2 * vt2) / det;
        let yyt = (u1 * vt2 - v1 * ut2) / det;
        //
        let xt = move |x: f32, y: f32| xxt * x + yxt * y + texcoords[top].0;
        let yt = move |x: f32, y: f32| xyt * x + yyt * y + texcoords[top].1;
        (xt, yt)
    };
    //
    // println!("{}", xt(0.0, 0.0));
    //
    {
        let leftm = {
            let dx = coords[bot].0 - coords[top].0;
            let dy = coords[bot].1 - coords[top].1;
            dx / dy
        };
        let xtop = coords[top].0.ceil();
        let ytop = coords[top].1.ceil() + 0.5;
        let ymid = coords[mid].1.ceil() + 0.5;
        let ybot = coords[bot].1.ceil() + 0.5;
        let mut y = ytop;
        //
        let mirrored = leftm * (ymid - ytop) + coords[top].0 > coords[mid].0;
        //
        {
            let rightm = {
                let dx = coords[mid].0 - coords[top].0;
                let dy = coords[mid].1 - coords[top].1;
                dx / dy
            };
            while y < ymid {
                let leftx = (leftm * (y - ytop) + coords[top].0) as usize;
                let rightx = (rightm * (y - ytop) + coords[top].0) as usize;
                let (leftx, rightx) = if mirrored {
                    (rightx, leftx)
                } else {
                    (leftx, rightx)
                };
                let yi = y as usize;
                let mut x = leftx as f32;
                for xi in leftx..rightx + 1 {
                    buffer[screencoords_to_ind(xi, yi, w)] = texcoord_to_col(
                        texture,
                        wt,
                        ht,
                        xt(x - xtop, y - ytop),
                        yt(x - xtop, y - ytop),
                    );
                    x += 1.0;
                }
                y += 1.0;
            }
        }
        //
        {
            let rightm = {
                let dx = coords[bot].0 - coords[mid].0;
                let dy = coords[bot].1 - coords[mid].1;
                dx / dy
            };
            while y < ybot {
                let leftx = (leftm * (y - ytop) + coords[top].0) as usize;
                let rightx = (rightm * (y - ymid) + coords[mid].0) as usize;
                let (leftx, rightx) = if mirrored {
                    (rightx, leftx)
                } else {
                    (leftx, rightx)
                };
                let yi = y as usize;
                let mut x = leftx as f32;
                for xi in leftx..rightx + 1 {
                    buffer[screencoords_to_ind(xi, yi, w)] = texcoord_to_col(
                        texture,
                        wt,
                        ht,
                        xt(x - xtop, y - ytop),
                        yt(x - xtop, y - ytop),
                    );
                    x += 1.0;
                }
                y += 1.0;
            }
        }
    }
}

fn main() {
    const W: usize = 64 * 8;
    const H: usize = 64 * 8;
    let options = WindowOptions {
        borderless: false,
        title: true,
        resize: false,
        scale: Scale::X1,
        scale_mode: ScaleMode::Stretch,
    };
    let mut window =
        Window::new("Test", W, H, options).expect("Unable to open Window");
    //
    // window.limit_update_rate(Some(Duration::from_secs_f64(1.0 / 60.0)));
    //
    let mut buffer = vec![0; W * H];
    //
    let mut triangle1 = [(-0.8, -0.8), (0.8, -0.8), (0.8, 0.8)];
    let texcoords1 = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)];
    // let triangle2 = [(-0.8, -0.8), (-0.8, 0.8), (0.8, 0.8)];
    // let texcoords2 = [(0.0, 0.0), (0.0, 1.0), (1.0, 1.0)];
    //
    let img: Vec<_> = image::open("res/mandrill.png")
        .unwrap()
        .to_bgra()
        .iter()
        .tuples()
        .map(|(b, g, r, a)| argb_to_col(*a, *r, *g, *b))
        .collect();
    //
    while window.is_open() {
        for col in buffer.iter_mut() {
            *col = 0;
        }
        //
        render_triangle(
            &mut buffer,
            &triangle1,
            &texcoords1,
            W,
            H,
            &img,
            512,
            512,
        );
        //
        // render_triangle(
        //     &mut buffer,
        //     &triangle2,
        //     &texcoords2,
        //     W,
        //     H,
        //     &img,
        //     512,
        //     512,
        // );
        //
        if let Some((x, y)) = window.get_mouse_pos(MouseMode::Pass) {
            if x >= 0.0 && x < W as f32 && y >= 0.0 && y < H as f32 {
                let ind = if window.get_mouse_down(MouseButton::Left) {
                    Some(0)
                } else if window.get_mouse_down(MouseButton::Right) {
                    Some(1)
                } else if window.get_mouse_down(MouseButton::Middle) {
                    Some(2)
                } else {
                    None
                };
                if let Some(ind) = ind {
                    triangle1[ind] = screencoords_to_xy(x, y, W, H);
                }
            }
        }
        //
        window.update_with_buffer(&buffer, W, H).unwrap();
    }
}
