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
use std::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use log::debug;
use sliding_features::{Echo, View, ALMA};

use crate::{connect::Connection, fps::Fps, Message};

pub fn jank(rx: &Receiver<Message>) {
    debug!("Connecting to root process");
    let mut connection = Connection::init_and_wait().unwrap(); // 等待root程序链接
    debug!("Connected");

    let mut vsync_stamp = Instant::now();
    let mut soft_stamp = Instant::now();

    let mut vsync_fps = Fps::default();

    let target_fps = connection.get_input().unwrap_or_default();

    let win = target_fps.fps / 10;
    let win = win.min(5);

    let mut alma = ALMA::new(Echo::new(), win.try_into().unwrap());

    loop {
        let target_fps_up = connection.get_input().unwrap_or_default();

        if target_fps != target_fps_up {
            let win = target_fps.fps / 10;
            let win = win.min(5);
            alma = ALMA::new(Echo::new(), win.try_into().unwrap());
            continue;
        }

        let message = rx.recv().unwrap();
        let now = Instant::now();

        let soft_fps = match message {
            Message::Vsync => {
                vsync_fps = Fps::from_frametime(now - vsync_stamp);
                vsync_stamp = now;
                continue;
            }
            Message::Soft => {
                let fps = Fps::from_frametime(now - soft_stamp);
                soft_stamp = now;
                fps
            }
        };

        alma.update(soft_fps.frametime.as_secs_f64());
        let cur_frametime_ns = alma.last();
        let target_frametime_ns = target_fps.frametime.as_secs_f64();

        debug!("cur frametime: {cur_frametime_ns}");
        debug!("target fps: {target_frametime_ns}");
        debug!("vsync_fps: {vsync_fps:?}");

        let diff_ns = cur_frametime_ns - target_frametime_ns;

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        if diff_ns < 0.0 {
            connection.send_jank(0);
        } else {
            let level = diff_ns / Duration::from_nanos(100_000).as_secs_f64().floor();
            connection.send_jank(level as u32);
        }
    }
}
