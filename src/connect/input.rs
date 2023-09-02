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
use std::{fs, time::Duration};

use log::error;

use super::Connection;
use crate::{
    error::{Error, Result},
    fps::Fps,
};

impl Connection {
    pub fn get_input(&mut self) -> Option<(Fps, Duration)> {
        let input_raw = fs::read_to_string(&self.input_path).unwrap();
        if input_raw == self.input_raw {
            return self.input;
        }

        self.input_raw = input_raw;
        Self::parse_input(&self.input_raw).map_or_else(|e| error!("{e:?}"), |i| self.input = i);

        self.input
    }

    pub fn parse_input<S: AsRef<str>>(i: S) -> Result<Option<(Fps, Duration)>> {
        let input = i.as_ref().trim();

        if input.is_empty() {
            return Err(Error::Other("No input now"));
        }

        if input.contains("none") {
            return Ok(None);
        }

        let mut iter = input.trim().split(':');

        let input = iter.next();
        let fps = input
            .ok_or(Error::Other("Failed to parse input"))?
            .trim()
            .parse()
            .map_err(|_| Error::Other("Failed to parse input"))?;

        let input = iter.next();
        let fix_time = input
            .ok_or(Error::Other("Failed to parse input"))?
            .trim()
            .parse()
            .map_err(|_| Error::Other("Failed to parse input"))?;
        let fix_time = Duration::from_nanos(fix_time);

        Ok(Some((Fps::from_fps(fps), fix_time)))
    }
}
