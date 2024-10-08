use crate::prelude::*;
use theframework::prelude::*;

/// 2D hash, taken from https://www.shadertoy.com/view/4djSRW
#[inline(always)]
pub fn hash21(p: Vec2f) -> f32 {
    let mut p3 = frac(vec3f(p.x * 0.1031, p.y * 0.1031, p.x * 0.1031));
    let dot = dot(p3, vec3f(p3.y + 33.333, p3.z + 33.333, p3.x + 33.333));

    p3.x += dot;
    p3.y += dot;
    p3.z += dot;
    ((p3.x + p3.y) * p3.z).fract()
}

pub fn steepness(coll: &TheCollection, _uv: Vec2f, hit: &mut Hit) -> (u8, u8) {
    let angle1 = coll.get_f32_default("Angle #1", 10.0);
    let angle2 = coll.get_f32_default("Angle #2", 30.0);

    let slope_angle = acos(dot(hit.normal, vec3f(0.0, 1.0, 0.0)));

    if slope_angle < angle1.to_radians() {
        (0, 0)
    } else if slope_angle < angle2.to_radians() {
        (1, 1)
    } else {
        (2, 2)
    }
}

pub fn subdivide(coll: &TheCollection, uv: Vec2f, hit: &mut Hit) -> u8 {
    let mode = coll.get_i32_default("Mode", 0);
    let offset = coll.get_f32_default("Offset", 0.5);

    if mode == 0 {
        if uv.x < offset {
            hit.uv = uv / offset;
            0
        } else {
            hit.uv = (uv - vec2f(offset, 0.0)) / (1.0 - offset);
            1
        }
    } else if uv.y < offset {
        hit.uv = uv / offset;
        0
    } else {
        hit.uv = (uv - vec2f(0.0, offset)) / (1.0 - offset);
        1
    }
}

pub fn noise2d(p: &Vec2f, scale: Vec2f, octaves: i32) -> f32 {
    fn hash(p: Vec2f) -> f32 {
        let mut p3 = frac(vec3f(p.x, p.y, p.x) * 0.13);
        p3 += dot(p3, vec3f(p3.y, p3.z, p3.x) + 3.333);
        frac((p3.x + p3.y) * p3.z)
    }

    fn noise(x: Vec2f) -> f32 {
        let i = floor(x);
        let f = frac(x);

        let a = hash(i);
        let b = hash(i + vec2f(1.0, 0.0));
        let c = hash(i + vec2f(0.0, 1.0));
        let d = hash(i + vec2f(1.0, 1.0));

        let u = f * f * (3.0 - 2.0 * f);
        lerp(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y
    }

    // let scale = vec2f(
    //     coll.get_f32_default("UV Scale X", 1.0),
    //     coll.get_f32_default("UV Scale Y", 1.0),
    // );
    // let octaves = coll.get_i32_default("Octaves", 5);

    let mut x = *p * 8.0 * scale;

    if octaves == 0 {
        return noise(x);
    }

    let mut v = 0.0;
    let mut a = 0.5;
    let shift = vec2f(100.0, 100.0);
    // Rotate to reduce axial bias
    let rot = Mat2::new(cos(0.5), sin(0.5), -sin(0.5), cos(0.50));
    for _ in 0..octaves {
        v += a * noise(x);
        x = rot * x * 2.0 + shift;
        a *= 0.5;
    }
    v
}

pub fn noise3d(coll: &TheCollection, p: &Vec3f) -> f32 {
    fn hash(mut p: f32) -> f32 {
        p = frac(p * 0.011);
        p *= p + 7.5;
        p *= p + p;
        frac(p)
    }

    fn noise(x: Vec3f) -> f32 {
        let step: Vec3f = vec3f(110.0, 241.0, 171.0);

        let i = floor(x);
        let f = frac(x);

        let n = dot(i, step);

        let u = f * f * (3.0 - 2.0 * f);
        lerp(
            lerp(
                lerp(
                    hash(n + dot(step, vec3f(0.0, 0.0, 0.0))),
                    hash(n + dot(step, vec3f(1.0, 0.0, 0.0))),
                    u.x,
                ),
                lerp(
                    hash(n + dot(step, vec3f(0.0, 1.0, 0.0))),
                    hash(n + dot(step, vec3f(1.0, 1.0, 0.0))),
                    u.x,
                ),
                u.y,
            ),
            lerp(
                lerp(
                    hash(n + dot(step, vec3f(0.0, 0.0, 1.0))),
                    hash(n + dot(step, vec3f(1.0, 0.0, 1.0))),
                    u.x,
                ),
                lerp(
                    hash(n + dot(step, vec3f(0.0, 1.0, 1.0))),
                    hash(n + dot(step, vec3f(1.0, 1.0, 1.0))),
                    u.x,
                ),
                u.y,
            ),
            u.z,
        )
    }

    let scale = vec3f(
        coll.get_f32_default("UV Scale X", 1.0),
        coll.get_f32_default("UV Scale Y", 1.0),
        coll.get_f32_default("UV Scale Z", 1.0),
    );

    let octaves = coll.get_i32_default("Octaves", 5);

    let mut x = 1240.0 + *p * 8.0 * scale;

    if octaves == 0 {
        return noise(x);
    }

    let mut v = 0.0;
    let mut a = 0.5;
    let shift = vec3f(100.0, 100.0, 100.0);
    for _ in 0..octaves {
        v += a * noise(x);
        x = x * 2.0 + shift;
        a *= 0.5;
    }
    v
}

fn rot(a: f32) -> Mat2f {
    Mat2f::new(a.cos(), -a.sin(), a.sin(), a.cos())
}

// Shane's box divide formula from https://www.shadertoy.com/view/XsGyDh
pub fn box_divide(p: Vec2f, gap: f32, rotation: f32, rounding: f32) -> (f32, f32) {
    fn s_box(p: Vec2f, b: Vec2f, r: f32) -> f32 {
        let d = abs(p) - b + vec2f(r, r);
        d.x.max(d.y).min(0.0) + length(max(d, vec2f(0.0, 0.0))) - r
    }

    let mut p = p;
    let ip = floor(p);
    p -= ip;
    let mut l = vec2f(1.0, 1.0);
    let mut last_l;
    let mut r = hash21(ip);

    for _ in 0..6 {
        r = (dot(l + vec2f(r, r), vec2f(123.71, 439.43)).fract() * 0.4) + (1.0 - 0.4) / 2.0;

        last_l = l;
        if l.x > l.y {
            p = vec2f(p.y, p.x);
            l = vec2f(l.y, l.x);
        }

        if p.x < r {
            l.x /= r;
            p.x /= r;
        } else {
            l.x /= 1.0 - r;
            p.x = (p.x - r) / (1.0 - r);
        }

        if last_l.x > last_l.y {
            p = vec2f(p.y, p.x);
            l = vec2f(l.y, l.x);
        }
    }
    p -= 0.5;

    // Create the id
    let id = hash21(ip + l);

    // Slightly rotate the tile based on its id
    p = rot((id - 0.5) * rotation) * p;

    // Gap, or mortar, width. Using "l" to keep it roughly constant.
    let th = l * 0.02 * gap;
    // Take the subdivided space and turn them into rounded pavers.
    //let c = s_box(p, vec2f(0.5, 0.5) - th, noise(p) * 0.5);
    let c = s_box(p, vec2f(0.5, 0.5) - th, rounding);
    // Smoothing factor.
    //let sf = 2.0 / res.x * length(l);
    // Individual tile ID.

    // Return distance and id
    (c, id)
}

pub fn bricks(uv: Vec2f, hit: &mut Hit, params: &[f32]) -> f32 {
    fn s_box(p: Vec2f, b: Vec2f, r: f32) -> f32 {
        let d = abs(p) - b + vec2f(r, r);
        d.x.max(d.y).min(0.0) + length(max(d, vec2f(0.0, 0.0))) - r
    }

    let ratio = params[0];
    let round = params[1];
    let rotation = params[2];
    let gap = params[3];
    let cell = params[4];
    let mode = params[5] as i32;

    let mut u = uv;

    let w = vec2f(ratio, 1.0);
    u *= vec2f(cell, cell) / w;

    if mode == 0 {
        u.x += 0.5 * u.y.floor() % 2.0;
    }

    let id = hash21(floor(u));

    let mut p = frac(u);
    p = rot((id - 0.5) * rotation) * (p - 0.5);

    hit.hash = id;
    hit.uv = p;

    s_box(p, vec2f(0.5, 0.5) - gap, round)
}
