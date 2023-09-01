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
use yata::{methods::DEMA, prelude::*};

use crate::{connect::Connection, fps::Fps, Message};

pub fn jank(rx: &Receiver<Message>) {
    debug!("Connecting to root process");
    let mut connection = Connection::init_and_wait().unwrap(); // 等待root程序链接
    debug!("Connected");

    let mut vsync_stamp = Instant::now();
    let mut soft_stamp = Instant::now();

    let mut vsync_fps = Fps::default();

    let mut dema = DEMA::new(5, &0.0).unwrap();

    loop {
        let target_fps = connection.get_input().unwrap_or_default();

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

        let soft_fps = Fps::from_frametime(Duration::from_secs_f64(
            dema.next(&soft_fps.frametime.as_secs_f64()),
        ));

        debug!("cur fps: {soft_fps:?}");
        debug!("target fps: {target_fps:?}");
        debug!("vsync fps: {vsync_fps:?}");

        soft_fps
            .frametime
            .checked_sub(target_fps.frametime)
            .map_or_else(
                || connection.send_jank(0),
                |d| {
                    let level = d.as_nanos() / Duration::from_nanos(10000).as_nanos();
                    connection.send_jank(level.try_into().unwrap());
                },
            );
    }
}
