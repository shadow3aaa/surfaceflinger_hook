/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
use crate::Message;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Bound {
    pub vsync_jank_scale: u32, // 两次合成最多有多少个vsync
    pub soft_jank_scale: u32,  // vsync_do_scale次vsync最少要有几次合成
    pub vsync_do_scale: u32,   // 每vsync_do_scale次vsync结算一次jank
}

impl Bound {
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
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

#[test]
fn test_bound() {
    let answer = Bound {
        vsync_jank_scale: 1,
        soft_jank_scale: 1,
        vsync_do_scale: 1,
    };
    assert_eq!(Bound::new((120, 120, Message::Vsync)), answer);

    let answer = Bound {
        vsync_jank_scale: 1,
        soft_jank_scale: 1,
        vsync_do_scale: 1,
    };
    assert_eq!(Bound::new((120, 120, Message::Vsync)), answer);
}
