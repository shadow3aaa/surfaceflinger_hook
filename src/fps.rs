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
use std::{cmp, time::Duration};

use derive_more::{Add, Sub};

#[derive(Debug, PartialEq, Eq, Add, Sub, Default, Clone, Copy)]
pub struct Fps {
    pub fps: u32,
    pub frametime: Duration,
}

impl Fps {
    pub fn from_fps(f: u32) -> Self {
        let frametime = Duration::from_secs(1).checked_div(f).unwrap_or_default();

        Self { fps: f, frametime }
    }

    pub fn from_frametime(d: Duration) -> Self {
        let fps = Duration::from_secs(1).as_nanos() / d.as_nanos();

        Self {
            fps: fps.try_into().unwrap(),
            frametime: d,
        }
    }
}

impl Ord for Fps {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.frametime.cmp(&other.frametime)
    }
}

impl PartialOrd for Fps {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
