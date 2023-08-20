use std::sync::mpsc::Receiver;

use crate::{connect::Connection, Message};

pub fn jank(rx: &Receiver<Message>) {
    let mut count = 0;

    let connection = Connection::init_and_wait().unwrap(); // 等待root程序链接

    loop {
        let count_on = connection.required_count_on().unwrap();

        match rx.recv().unwrap() {
            Message::Vsync => {
                if Message::Vsync == count_on {
                    let _ = connection.send_count(Message::Vsync, count);
                    count = 0;
                } else {
                    count += 1;
                }
            }
            Message::Soft => {
                if Message::Soft == count_on {
                    let _ = connection.send_count(Message::Soft, count);
                    count = 0;
                } else {
                    count += 1;
                }
            }
        }
    }
}
