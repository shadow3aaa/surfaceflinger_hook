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

pub fn jank(rx: &Receiver<()>) {
    debug!("Connecting to root process");

    let mut connection = Connection::init_and_wait().unwrap();

    debug!("Connected to root process");

    let mut soft_stamp = Instant::now();
    let (mut target_fps, _) = connection.get_input().unwrap_or_default();

    let win = target_fps.fps / 6;
    let win = win.max(5);
    let mut alma = ALMA::new(Echo::new(), win.try_into().unwrap_or(5));

    loop {
        rx.recv().unwrap();

        let (target_fps_up, fix_time) = connection.get_input().unwrap_or_default();

        let fix_time = fix_time.min(target_fps.frametime);

        if target_fps != target_fps_up {
            target_fps = target_fps_up;

            let win = target_fps.fps / 6;
            let win = win.max(5);
            alma = ALMA::new(Echo::new(), win.try_into().unwrap_or(5));

            continue;
        }

        let now = Instant::now();
        let soft_fps = Fps::from_frametime(now - soft_stamp);
        soft_stamp = now;

        alma.update(soft_fps.frametime.as_secs_f64());
        let cur_frametime = alma.last();
        let target_frametime = target_fps.frametime.as_secs_f64();

        debug!("cur frametime: {cur_frametime}");
        debug!("target fps: {target_frametime}");

        let diff = cur_frametime - target_frametime;

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        if diff < 0.0 {
            connection.send_jank(0);
        } else {
            let level = if diff <= 0.0 + fix_time.as_secs_f64() {
                0 // no jank
            } else if diff <= target_frametime / 10.0 + fix_time.as_secs_f64() {
                1 // simp jank
            } else if diff <= target_frametime / 5.0 + fix_time.as_secs_f64() {
                2 // big jank
            } else {
                4 // heavy jank
            };

            connection.send_jank(level);
        }
    }
}
