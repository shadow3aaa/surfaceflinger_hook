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
// mod bound;
mod input;

use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use unix_named_pipe as named_pipe;

use crate::{
    error::{Error, Result}, API_DIR, fps::Fps,
};

pub struct Connection {
    input: Fps, // target_fps
    input_raw: String,
    input_path: PathBuf,
    jank_path: PathBuf,
}

impl Connection {
    // 初始化和root程序的连接，堵塞，放在HookThread处理
    pub fn init_and_wait() -> Result<Self> {
        // 初始化compose count管道
        let jank_path = Path::new(API_DIR).join("jank");
        let input_path = Path::new(API_DIR).join("input");

        named_pipe::create(&jank_path, Some(0o644)).map_err(|_| Error::NamedPipe)?;
        fs::write(&input_path, "")?;

        let _ = OpenOptions::new().write(true).open(&jank_path)?; // 确认连接

        let (input, input_raw) = loop {
            let temp = fs::read_to_string(&input_path)?; // 等待root程序通过api初始化input，同时在此处与api确认连接
            if let Some(input) = Self::parse_input(&temp) {
                break (input, temp);
            }
            thread::sleep(Duration::from_secs(1));
        };

        Ok(Self {
            input,
            input_raw,
            input_path,
            jank_path,
        })
    }

    pub fn send_jank(&self, j: u32) {
        let message = format!("{j}\n");
        let _ = fs::write(&self.jank_path, message);
    }
}
