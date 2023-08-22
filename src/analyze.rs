use std::sync::mpsc::Receiver;

use crate::{connect::Connection, Message};

pub fn jank(rx: &Receiver<Message>) {
    let mut connection = Connection::init_and_wait().unwrap(); // 等待root程序链接

    loop {
        let message = rx.recv().unwrap();
        connection.notice(message);
    }
}
