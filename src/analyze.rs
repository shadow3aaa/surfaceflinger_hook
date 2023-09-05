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
use std::{sync::mpsc::Receiver, time::Instant};

use log::debug;
use sliding_features::{Echo, View, ALMA};

use crate::{connect::Connection, fps::Fps};

const SMOOTH_WIN: usize = 5;

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
pub fn jank(rx: &Receiver<()>) {
    debug!("Connecting to root process");

    let mut connection = Connection::init_and_wait().unwrap();

    debug!("Connected to root process");

    let mut soft_stamp = Instant::now();
    let (mut target_fps, _) = connection.get_input().unwrap_or_default();

    let mut smooth_frametime = ALMA::new(Echo::new(), SMOOTH_WIN);
    smooth_frametime.update(target_fps.frametime.as_nanos() as f64);

    let mut min_jank_scale = target_fps
        .frametime
        .checked_div(target_fps.fps)
        .unwrap_or_default()
        .as_nanos() as f64;

    loop {
        rx.recv().unwrap();

        let (target_fps_up, fix_time) = connection.get_input().unwrap_or_default();

        let fix_time = fix_time.min(target_fps.frametime).as_nanos() as f64;

        if target_fps != target_fps_up {
            target_fps = target_fps_up;

            smooth_frametime = ALMA::new(Echo::new(), SMOOTH_WIN);
            smooth_frametime.update(target_fps.frametime.as_nanos() as f64);

            min_jank_scale = target_fps
                .frametime
                .checked_div(target_fps.fps)
                .unwrap_or_default()
                .as_nanos() as f64;

            continue;
        }

        let now = Instant::now();
        let soft_fps = Fps::from_frametime(now - soft_stamp);
        soft_stamp = now;
        smooth_frametime.update(soft_fps.frametime.as_nanos() as f64);

        let cur_frametime = smooth_frametime.last();
        let target_frametime = target_fps.frametime.as_nanos() as f64;

        debug!("cur frametime: {cur_frametime} ns");
        debug!("target fps: {target_frametime} ns");

        let diff = cur_frametime - target_frametime;

        let level = if diff <= min_jank_scale + fix_time {
            0 // no jank
        } else if diff <= min_jank_scale.mul_add(2.0, fix_time) {
            1 // simp jank
        } else if diff <= min_jank_scale.mul_add(3.0, fix_time) {
            3 // big jank
        } else if diff <= min_jank_scale.mul_add(5.0, fix_time) {
            4 // heavy jank
        } else {
            8 // super jank
        };

        connection.send_jank(level);
    }
}
