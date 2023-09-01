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
use std::fs;

use super::Connection;
use crate::fps::Fps;

impl Connection {
    pub fn update_input(&mut self, d: Fps) {
        let input_raw = fs::read_to_string(&self.input_path).unwrap();
        if input_raw == self.input_raw {
            return;
        }

        self.input_raw = input_raw;
        self.input = Self::parse_input(&self.input_raw).unwrap_or(d);
    }

    pub fn parse_input<S: AsRef<str>>(i: S) -> Option<Fps> {
        let input = i.as_ref().trim();

        if input.is_empty() {
            return None;
        }

        let target_fps = input.split(':').last().and_then(|t| {
            if t.contains("none") {
                None
            } else {
                t.parse::<u32>().ok()
            }
        })?;

        Some(Fps::from_fps(target_fps))
    }
}
