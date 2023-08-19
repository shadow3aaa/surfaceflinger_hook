use std::{
    collections::VecDeque,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use log::info;

const OTHERS_BUFFER_SIZE: usize = 128;
const TARGET_FPS_BUFFER_SIZE: usize = 20;

pub enum Message {
    Vsync,
    Soft,
}

pub fn jank(rx: &Receiver<Message>) {
    let mut frame_count = 0;
    loop {
        match rx.recv().unwrap() {
            Message::Vsync => {
                info!("Frame count {frame_count}");
                frame_count = 0;
            }
            Message::Soft => frame_count += 1,
        }
    }
}
