use std::{
    fs::{self, File},
    io::prelude::*,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use log::error;
use unix_named_pipe as named_pipe;

use crate::{error::Result, Message, API_DIR};

pub struct Connection {
    sx: Sender<(Message, u32)>,
    count_on: Message,
    count_on_path: PathBuf,
}

impl Connection {
    // 初始化和root程序的连接，堵塞，放在HookThread处理
    pub fn init_and_wait() -> Result<Self> {
        // 初始化compose count管道
        let path = Path::new(API_DIR).join("count");

        let _ = fs::remove_file(&path); // 删掉之前的管道
        named_pipe::create(&path, Some(0o644))?;

        let mut pipe = File::open(&path)?;

        let mut temp = String::new();
        let _ = pipe.read_to_string(&mut temp); // 等待root程序调用api写入一次任意内容来确认连接

        let (sx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("HookConnection".into())
            .spawn(move || Self::connection_thread(pipe, &rx))?;

        let count_on_path = Path::new(API_DIR).join("count_on");

        named_pipe::create(&count_on_path, Some(0o644))?; // vsync结算count和compose结算有各自的好处，这里给出接口供api控制何时结算
        let mut pipe = File::open(&count_on_path)?;

        let mut temp = String::new();
        pipe.read_to_string(&mut temp)?;

        let count_on = match temp.split('#').last().map(|s| s.trim()) {
            Some("vsync") => Message::Vsync,
            Some("soft") => Message::Soft,
            _ => {
                error!("Wrong inital message");
                error!("Use count-on vsync as default");
                Message::Vsync
            }
        };

        Ok(Self {
            sx,
            count_on,
            count_on_path,
        })
    }

    pub fn send_count(&self, m: Message, c: u32) -> Result<()> {
        self.sx.send((m, c))?;
        Ok(())
    }

    pub fn required_count_on(&self) -> Result<Message> {
        let mut pipe = named_pipe::open_read(&self.count_on_path)?; // 这会非堵塞的打开管道

        let mut r = String::new();

        if pipe.read_to_string(&mut r).is_ok() {
            match r.split('#').last().map(|s| s.trim()) {
                Some("vsync") => Ok(Message::Vsync),
                Some("soft") => Ok(Message::Soft),
                _ => {
                    error!("Wrong pipe message");
                    error!("Use last recorded value");
                    Ok(self.count_on)
                }
            }
        } else {
            Ok(self.count_on)
        }
    }

    fn connection_thread(mut pipe: File, rx: &Receiver<(Message, u32)>) {
        loop {
            let (m, c) = rx.recv().unwrap();
            let _ = match m {
                Message::Vsync => write!(pipe, "vsync:{c}#"),
                Message::Soft => write!(pipe, "soft:{c}#"),
            };
        }
    }
}
