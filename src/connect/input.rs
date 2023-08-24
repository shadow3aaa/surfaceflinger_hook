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
use std::{convert::AsRef, io::prelude::*};

use log::error;
use unix_named_pipe as named_pipe;

use crate::Message;

use super::{bound::Bound, Connection};

impl Connection {
    pub fn update_input(&mut self) {
        let Ok(mut pipe) = named_pipe::open_read(&self.input_path) else {
            return;
        }; // 非堵塞的打开管道，有新数据就更新

        let mut r = String::new();

        if pipe.read_to_string(&mut r).is_ok() {
            self.input = Self::parse_input(r);
            self.bound = Bound::new(self.input);
        }
    }

    pub fn parse_input<S: AsRef<str>>(i: S) -> (u32, u32, Message) {
        let input = i.as_ref();

        let Some(input) = input.lines().last() else {
            error!("Failed to parse input, use default");
            return (120, 120, Message::Vsync);
        };

        let mut input = input.split(':');

        let target_fps = input
            .next()
            .and_then(|i| i.parse::<u32>().ok())
            .unwrap_or_else(|| {
                error!("Failed to parse target_fps");
                120
            });

        let display_fps = input
            .next()
            .and_then(|i| i.parse::<u32>().ok())
            .unwrap_or_else(|| {
                error!("Failed to parse display_fps");
                120
            });

        let message = input
            .next()
            .and_then(|i| match i {
                "vsync" => Some(Message::Vsync),
                "soft" => Some(Message::Soft),
                _ => None,
            })
            .unwrap_or_else(|| {
                error!("Failed to parse count_on");
                Message::Vsync
            });

        (target_fps, display_fps, message)
    }
}
