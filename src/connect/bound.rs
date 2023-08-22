use crate::Message;

#[derive(Debug, Copy, Clone)]
pub struct Bound {
    pub vsync_jank_scale: u32, // 两次合成最多有多少个vsync
    pub soft_jank_scale: u32,  // vsync_do_scale次vsync最少要有几次合成
    pub vsync_do_scale: u32,   // 每vsync_do_scale次vsync结算一次jank
}

impl Bound {
    pub fn new(input: (u32, u32, Message)) -> Self {
        let soft_jank_scale = input.1 as f32 / input.0 as f32;
        let soft_jank_scale = soft_jank_scale.max(1.0).ceil() as u32;

        let (vsync_do_scale, vsync_jank_scale) = reduce_fraction(input.1, input.0);

        Self {
            vsync_jank_scale,
            soft_jank_scale,
            vsync_do_scale,
        }
    }
}

fn reduce_fraction(num: u32, den: u32) -> (u32, u32) {
    let gcd = gcd(num, den);

    let reduced_num = num / gcd;
    let reduced_den = den / gcd;

    (reduced_num, reduced_den)
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}
