//use crate::prelude::*;
use theframework::prelude::*;

pub fn bricks(coll: &TheCollection, uv: Vec2f) -> (u8, u8) {
    //let uv = hit.uv / 100.0;

    let ratio = coll.get_f32_default("Ratio", 2.0);
    let round = coll.get_f32_default("Rounding", 0.0);
    let bevel = coll.get_f32_default("Bevel", 0.0);
    let gap = coll.get_f32_default("Gap", 0.08);
    let mode = coll.get_i32_default("Mode", 0);

    let mut u = uv; // + vec2f(10000.0, 10000.0);

    let cell = coll.get_f32_default("Cell", 6.0);

    let w = vec2f(ratio, 1.0);
    u *= vec2f(cell, cell) / w; //u.component_mul(&FP2::new(self.cell, self.cell).component_div(&w));

    if mode == 0 {
        u.x += 0.5 * u.y.floor() % 2.0;
    }

    let t = frac(u) - vec2f(1.0, 1.0) / 2.0;
    let s = w * t;

    let a = w / 2.0 - gap - abs(s);
    let b = a * vec2f(2.0, 2.0) / bevel; ////a.component_mul(&FP2::new(2.0, 2.0)).component_div(&bevel);
    let mut m = b.x.min(b.y);
    if a.x < round && a.y < round {
        m = (round - length(vec2f(round, round) - a)) * 2.0; //
        dot(vec2f(bevel, bevel), normalize(vec2f(round, round) - a));
    }

    //(m.clamp(0.0, 1.0), self.hash21(glm::floor(&u)));

    let m = m.clamp(0.0, 1.0);

    if m == 1.0 {
        (0, 0)
    } else {
        (5, 1)
    }
}
