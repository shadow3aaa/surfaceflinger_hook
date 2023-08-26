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
use std::{convert::AsRef, fs};

use log::debug;

use crate::Message;

use super::{bound::Bound, Connection};

impl Connection {
    pub fn update_input(&mut self) {
        let input_raw = fs::read_to_string(&self.input_path).unwrap();

        if input_raw == self.input_raw {
            return;
        }

        if let Some(input) = Self::parse_input(&input_raw) {
            self.input = input;
            self.input_raw = input_raw;
            self.bound = Bound::new(self.input);
            debug!("{:#?}", self.bound);
        }
    }

    pub fn parse_input<S: AsRef<str>>(i: S) -> Option<(u32, u32, Message)> {
        let input = i.as_ref().trim();

        if input.is_empty() {
            return None;
        }

        let input = input.lines().last()?;

        let mut input = input.split(':');

        let target_fps = input.next().and_then(|i| i.parse::<u32>().ok())?;

        let display_fps = input.next().and_then(|i| i.parse::<u32>().ok())?;

        let message = input.next().and_then(|i| match i {
            "vsync" => Some(Message::Vsync),
            "soft" => Some(Message::Soft),
            _ => None,
        })?;

        Some((target_fps, display_fps, message))
    }
}
