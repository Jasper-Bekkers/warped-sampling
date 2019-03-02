use halton::Sequence;
use image::*;
use imageproc::drawing::*;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};

const WIDTH: usize = 1024;
const HEIGHT: usize = 1024;

fn into_buffer(image: &RgbaImage, buffer: &mut [u32]) {
    buffer
        .iter_mut()
        .zip(image.enumerate_pixels())
        .for_each(|(v, (_, _, rgba))| {
            *v = (rgba[2] as u32)
                | ((rgba[1] as u32) << 8)
                | ((rgba[0] as u32) << 16)
                | ((rgba[3] as u32) << 24);
        });
}

fn clear(image: &mut RgbaImage, clear_value: Rgba<u8>) {
    image
        .enumerate_pixels_mut()
        .for_each(|(_, _, v)| *v = clear_value);
}

#[derive(Debug, Copy, Clone)]
struct Box2f {
    min: (f64, f64),
    max: (f64, f64),
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

fn lerp_factor(a: f64, b: f64, m: f64) -> f64 {
    (m - a) / (b - a)
}

fn set_box(i: usize, edge: [[f64; 2]; 3]) -> Box2f {
    let ox = i & 1;
    let oy = (i & 2) >> 1;
    Box2f {
        min: (edge[0 + ox][0], edge[0 + oy][1]),
        max: (edge[1 + ox][0], edge[1 + oy][1]),
    }
}

fn warp_a_point(point: (f64, f64), texture_region: &Box2f, warp: &Box2f) -> (f64, f64) {
    let xf = lerp_factor(warp.min.0, warp.max.0, point.0);
    let yf = lerp_factor(warp.min.1, warp.max.1, point.1);

    (
        lerp(texture_region.min.0, texture_region.max.0, xf),
        lerp(texture_region.min.1, texture_region.max.1, yf),
    )
}

fn warp(
    mipmaps: &Vec<&Vec<Vec<f64>>>,
    level: usize,
    x: usize,
    y: usize,
    tx: usize,
    ty: usize,
    w: usize,
    scale: f64,
    b: &Box2f,
    points: &[(f64, f64)],
    warped_points: &mut Vec<(f64, f64)>,
    debug_boxes: &mut Vec<Box2f>,
) {
    if level >= mipmaps.len() {
        let tx = x;//dbg!(tx);
        let ty = y;//dbg!(ty);

        let inv_size = 1.0 / ((1 << level) as f64);

        let texture_box = Box2f {
            min: (0.5 * tx as f64 * inv_size, 0.5 * ty as f64 * inv_size),
            max: (0.5 * (tx as f64 + 2.0) * inv_size, 0.5 * (ty as f64 + 2.0) * inv_size),
        };

        debug_boxes.push(texture_box);

        for (x, y) in points {
            warped_points.push(warp_a_point((*x, *y), &texture_box, b))
        }

        return;
    }


    let cur_mip = &mipmaps[level];
    let pdfs = [
        scale * cur_mip[y][x],
        scale * cur_mip[y][x + 1],
        scale * cur_mip[y + 1][x],
        scale * cur_mip[y + 1][x + 1],
    ];


    let sum = 1.0 / (pdfs[0] + pdfs[1] + pdfs[2] + pdfs[3]);
    let mid_1 = sum * (pdfs[0] + pdfs[1]);
    let mid_0 = sum * pdfs[0] / mid_1;

    let mut edge = [
        [b.min.0, b.min.1],
        [lerp(b.min.0, b.max.0, mid_0), lerp(b.min.1, b.max.1, mid_1)],
        [b.max.0, b.max.1],
    ];

    let b0 = set_box(0, edge);
    let b1 = set_box(1, edge);

    let mid_2 = sum * pdfs[2] / (1.0 - mid_1);
    edge[1][0] = lerp(b.min.0, b.max.0, mid_2);

    let b2 = set_box(2, edge);
    let b3 = set_box(3, edge);

    let boxes = [b0, b1, b2, b3];

    let mut point_cache = [vec![], vec![], vec![], vec![]];

    for (x, y) in points {
        let mut offset = 0;
        offset += if *y > boxes[offset].max.1 { 2 } else { 0 };
        offset += if *x > boxes[offset].max.0 { 1 } else { 0 };
        point_cache[offset].push((*x, *y));
    }

    // dbg!(point_cache[0].len());
    // dbg!(point_cache[1].len());
    // dbg!(point_cache[2].len());
    // dbg!(point_cache[3].len());

    let nl = level + 1;
    let nx = x << 1;
    let ny = y << 1;
    let nw = w << 1;

    for idx in 0..4 {
        let ox = (idx & 1) ;
        let oy = ((idx & 2) >> 1) ;
        if point_cache[idx].len() > 0 {
            // let nl = dbg!(nl);
            // let nx = dbg!(nx);
            // let ox = dbg!(ox);
            // let ny = dbg!(ny);
            // let oy = dbg!(oy);
            warp(
                mipmaps,
                nl,
                nx + ox * 2,
                ny + oy * 2,
                nx + ox,
                ny + oy,
                nw,
                1.0,//pdfs[idx],
                &boxes[idx],
                &point_cache[idx],
                warped_points,
                debug_boxes,
            )
        }
    }
}

fn main() {
    let points = Sequence::new(2)
        .zip(Sequence::new(3))
        .take(10000)
        .collect::<Vec<_>>();

    let image_bytes = include_bytes!("../ImportanceSampleThis.png");

    let full_image = image::load_from_memory(image_bytes).unwrap();

    let mut mipmaps = vec![];

    mipmaps.push(full_image.to_rgba());

    loop
    {
        let img = mipmaps.last().unwrap();

        let nw = img.width() / 2;
        let nh = img.height() / 2;

        let mut ndata = vec![];

        for y in 0..nh {
            for x in 0..nw {
                let ox = x * 2;
                let oy = y * 2;

                let p0 = img.get_pixel(ox, oy).channels4();
                let p1 = img.get_pixel(ox + 1, oy).channels4();
                let p2 = img.get_pixel(ox, oy + 1).channels4();
                let p3 = img.get_pixel(ox + 1, oy + 1).channels4();

                ndata.extend_from_slice(&[
                    ((p0.0 as u32 + p1.0 as u32 + p2.0 as u32 + p3.0 as u32) / 4) as u8,
                    ((p0.1 as u32 + p1.1 as u32 + p2.1 as u32 + p3.1 as u32) / 4) as u8,
                    ((p0.2 as u32 + p1.2 as u32 + p2.2 as u32 + p3.2 as u32) / 4) as u8,
                    ((p0.3 as u32 + p1.3 as u32 + p2.3 as u32 + p3.3 as u32) / 4) as u8]);
            }
        }

        let nimg = ImageBuffer::from_vec(nw, nh, ndata).unwrap();
        mipmaps.push(nimg);
        
        if nw == 2 || nh == 2{
            break;
        }
    }

    println!("{}", mipmaps.len());

    // for (x, y) in &points {
    //     println!("{} {}", x, y);
    // }
    let background_image = &mipmaps[0];

    let mut window = Window::new("Warped sampling", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    let mut scratch_buffer = vec![0; WIDTH * HEIGHT];
    let mut image = RgbaImage::new(WIDTH as u32, HEIGHT as u32);

    let mipmaps = {
        let mut maps = vec![];
        for m in mipmaps.iter() {
            let mut pixels = vec![];
            for y in 0..m.height() {
                let mut row = vec![];
                for x in 0..m.width() {
                    row.push(m.get_pixel(x, y).channels4().0 as f64)
                }
                pixels.push(row);
            }
            maps.push(
                /*m.raw_pixels()
                .iter()
                .step_by(4)
                .map(|x| *x as f64 / 255.0 + 0.001)
                .collect::<Vec<_>>(),*/
                pixels,
            );
        }
        maps
    };

    for (midx, m) in mipmaps.iter().rev().enumerate() {
        println!("\nPixels {}:", midx);
        m.iter()
            .enumerate()
            .for_each(|(idx, r)| {
                let k = (1 << (1 + midx));

                if idx % 2 == 0 {
                    println!("{}", "-".repeat(k * 4 + (1 << midx) + 6));
                }

                print!("{:3} : ", idx);

                r.iter().enumerate().for_each(|(idx, p)| {
                    
                    print!("{:3} {}", p, if idx & 1 == 1  { "|" } else { "" })
                });


                println!("");
            });
        println!("");

        if midx >= 3 {
            break;
        }
    }

    let mipmaps = mipmaps
        .iter()
        .rev()
        //.skip(mipmaps.len() - 3)
        .collect::<Vec<_>>();

    let mut new_warped = vec![];
    let mut debug_boxes = vec![];
    warp(
        &mipmaps,
        0,
        0,0,0,
        0,
        2,
        1.0,
        &Box2f {
            min: (0.0, 0.0),
            max: (1.0, 1.0),
        },
        &points,
        &mut new_warped,
        &mut debug_boxes,
    );

    while window.is_open() && !window.is_key_down(Key::Escape) {
        clear(&mut image, Rgba([0, 0, 255, 255]));
        imageops::replace(&mut image, &background_image, 0, 0);

    if false
        {
            for (x, y) in &points {
                draw_hollow_circle_mut(
                    &mut image,
                    (
                        (x.max(0.0) * background_image.width() as f64) as i32,
                        (y.max(0.0) * background_image.height() as f64) as i32,
                    ),
                    2,
                    Rgba([0xee, 0, 0, 0x33]),
                );
            }
        }

            for (x, y) in &new_warped {
                draw_hollow_circle_mut(
                    &mut image,
                    (
                        (x.max(0.0) * background_image.width() as f64) as i32,
                        (y.max(0.0) * background_image.height() as f64) as i32,
                    ),
                    2,
                    Rgba([0x0, 0xee, 0, 0x33]),
                );
            }

        for b in debug_boxes.iter() {
            draw_hollow_rect_mut(
                &mut image,
                imageproc::rect::Rect::at((b.min.0 * background_image.width() as f64) as i32, (b.min.1 * background_image.height() as f64) as i32)
                    .of_size(
                        ((b.max.0 - b.min.0) * background_image.width() as f64) as u32,
                        ((b.max.1 - b.min.1) * background_image.height() as f64) as u32,
                    ),
                Rgba([0xff, 0xff, 0x44, 0]),
            );
        }

        // for (x, y) in &points {
        //     draw_hollow_circle_mut(
        //         &mut image,
        //         ((x * WIDTH as f64) as i32, (y * HEIGHT as f64) as i32),
        //         4,
        //         Rgba([0x0, 0xee, 0xee, 0x33]),
        //     );
        // }

        into_buffer(&image, &mut scratch_buffer);
        window.update_with_buffer(&scratch_buffer).unwrap();
    }
}
