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

pub fn warp(mipmaps: &Vec<&Vec<Vec<f64>>>, points: &[(f64, f64)], warped_points: &mut Vec<(f64, f64)>) {
    warp_recurse(
        &mipmaps,
        0,
        0,
        0,
        2,
        1.0,
        &Box2f {
            min: (0.0, 0.0),
            max: (1.0, 1.0),
        },
        &points,
        warped_points,
    );
}

fn warp_recurse(
    mipmaps: &Vec<&Vec<Vec<f64>>>,
    level: usize,
    x: usize,
    y: usize,
    w: usize,
    scale: f64,
    b: &Box2f,
    points: &[(f64, f64)],
    warped_points: &mut Vec<(f64, f64)>,
) {
    if level >= mipmaps.len() {
        let inv_size = 1.0 / ((1 << level + 1) as f64);

        let texture_box = Box2f {
            min: (x as f64 * inv_size, y as f64 * inv_size),
            max: ((x as f64 + 2.0) * inv_size, (y as f64 + 2.0) * inv_size),
        };

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

    let nl = level + 1;
    let nx = x << 1;
    let ny = y << 1;
    let nw = w << 1;

    for idx in 0..4 {
        let ox = (idx & 1) << 1;
        let oy = idx & 2;
        if point_cache[idx].len() > 0 {
            warp_recurse(
                mipmaps,
                nl,
                nx + ox,
                ny + oy,
                nw,
                pdfs[idx],
                &boxes[idx],
                &point_cache[idx],
                warped_points,
            )
        }
    }
}
