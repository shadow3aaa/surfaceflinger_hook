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
use std::sync::mpsc::Receiver;

use crate::{connect::Connection, Message};

// Todo: 目前只做不堵塞surfaceflinger以等待api链接用，应修改connection优化掉此线程
pub fn jank(rx: &Receiver<Message>) {
    let mut connection = Connection::init_and_wait().unwrap(); // 等待root程序链接

    loop {
        let message = rx.recv().unwrap();
        connection.notice(message);
    }
}
