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
mod bound;
mod input;

use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use unix_named_pipe as named_pipe;

use crate::{
    error::{Error, Result},
    Message, API_DIR,
};
use bound::Bound;

pub struct Connection {
    sx: Sender<u32>,
    input: (u32, u32, Message), // target_fps:display_fps:count_on
    input_path: PathBuf,
    bound: Bound,
    vsync_count: u32,
    soft_count: u32,
}

impl Connection {
    // 初始化和root程序的连接，堵塞，放在HookThread处理
    pub fn init_and_wait() -> Result<Self> {
        // 初始化compose count管道
        let jank_path = Path::new(API_DIR).join("jank");
        let input_path = Path::new(API_DIR).join("input");

        // 目前hook文件夹在/dev(tmpfs)下，重启不保留，删除无意义
        /* let _ = fs::remove_file(&jank_path);
        let _ = fs::remove_file(&input_path); */

        named_pipe::create(&jank_path, Some(0o644)).map_err(|_| Error::NamedPipe)?;
        named_pipe::create(&input_path, Some(0o644)).map_err(|_| Error::NamedPipe)?;

        let _ = OpenOptions::new().read(true).open(&input_path)?;
        let _ = OpenOptions::new().write(true).open(&jank_path)?; // 确认连接

        let (sx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("HookConnection".into())
            .spawn(move || Self::connection_thread(&jank_path, &rx))?;

        let temp = fs::read_to_string(&input_path)?; // 等待root程序通过api初始化input，同时在此处与api确认连接
        let input = Self::parse_input(temp);
        let bound = Bound::new(input);

        Ok(Self {
            sx,
            input,
            input_path,
            bound,
            vsync_count: 0,
            soft_count: 0,
        })
    }

    pub fn notice(&mut self, m: Message) {
        let count_on = self.input.2;

        match count_on {
            Message::Vsync => {
                if Message::Vsync == m && self.vsync_count >= self.bound.vsync_do_scale {
                    let min = self.bound.soft_jank_scale;
                    let jank_level = self.soft_count.saturating_sub(min);

                    let _ = self.sx.send(jank_level);

                    self.soft_count = 0;
                    self.vsync_count = 0;
                }
            }
            Message::Soft => {
                let max = self.bound.vsync_jank_scale;
                let jank_level = max.saturating_sub(self.vsync_count);

                let _ = self.sx.send(jank_level);

                self.soft_count = 0;
                self.vsync_count = 0;
            }
        }

        match m {
            Message::Vsync => self.vsync_count += 1,
            Message::Soft => self.soft_count += 1,
        }

        self.update_input();
    }

    fn connection_thread(pipe: &Path, rx: &Receiver<u32>) {
        loop {
            let _level = rx.recv().unwrap();
            let _ = fs::write(pipe, "{level}\n");
        }
    }
}
