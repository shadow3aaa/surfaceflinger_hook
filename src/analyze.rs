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
