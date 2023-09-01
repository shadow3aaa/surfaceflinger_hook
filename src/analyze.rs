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
use std::{sync::mpsc::Receiver, time::{Instant, Duration}};

use crate::{connect::Connection, Message, fps::Fps};

// Todo: 目前只做不堵塞surfaceflinger以等待api链接用，应修改connection优化掉此线程
pub fn jank(rx: &Receiver<Message>) {
    let mut connection = Connection::init_and_wait().unwrap(); // 等待root程序链接

    let mut vsync_stamp = Instant::now();
    let mut soft_stamp = Instant::now();

    let mut vsync_fps = Fps::default();

    loop {
        connection.update_input(vsync_fps);

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
        
        match vsync_fps.frametime.checked_sub(soft_fps.frametime) {
            Some(d) => {
                let level = d.as_nanos() / Duration::from_nanos(750).as_nanos();
                connection.send_jank(level.try_into().unwrap())
            },
            None => connection.send_jank(0),
        }
    }
}
