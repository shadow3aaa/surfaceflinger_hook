mod bound;

use std::{
    convert::AsRef,
    fs::{self, File},
    io::prelude::*,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use log::error;
use unix_named_pipe as named_pipe;

use crate::{
    error::{Error, Result},
    Message, API_DIR,
};
use bound::Bound;

pub struct Connection {
    sx: Sender<u32>,
    input: (u32, u32, Message), // target_fps:display_fps:count_on
    input_path: PathBuf,
    bound: Bound,
    vsync_count: u32,
    soft_count: u32,
}

impl Connection {
    // 初始化和root程序的连接，堵塞，放在HookThread处理
    pub fn init_and_wait() -> Result<Self> {
        // 初始化compose count管道
        let jank_path = Path::new(API_DIR).join("jank");
        let input_path = Path::new(API_DIR).join("input");

        let _ = fs::remove_file(&jank_path);
        let _ = fs::remove_file(&input_path);

        named_pipe::create(&jank_path, Some(0o644)).map_err(|_| Error::NamedPipe)?;
        named_pipe::create(&input_path, Some(0o644)).map_err(|_| Error::NamedPipe)?;

        let output_pipe = File::open(&jank_path)?;
        let (sx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("HookConnection".into())
            .spawn(move || Self::connection_thread(output_pipe, &rx))?;

        let mut input_pipe = File::open(&input_path)?;

        let mut temp = String::new();
        input_pipe.read_to_string(&mut temp)?; // 等待root程序通过api初始化input，同时在此处与api确认连接

        let input = Self::parse_input(temp);
        let bound = Bound::new(input);

        Ok(Self {
            sx,
            input,
            input_path,
            bound,
            vsync_count: 0,
            soft_count: 0,
        })
    }

    pub fn notice(&mut self, m: Message) {
        let count_on = self.input.2;

        match count_on {
            Message::Vsync => {
                if Message::Vsync == m && self.vsync_count >= self.bound.vsync_do_scale {
                    let min = self.bound.soft_jank_scale;
                    let jank_level = self.soft_count.saturating_sub(min);

                    let _ = self.sx.send(jank_level);

                    self.soft_count = 0;
                    self.vsync_count = 0;
                }
            }
            Message::Soft => {
                let max = self.bound.vsync_jank_scale;
                let jank_level = max.saturating_sub(self.vsync_count);

                let _ = self.sx.send(jank_level);

                self.soft_count = 0;
                self.vsync_count = 0
            }
        }

        match m {
            Message::Vsync => self.vsync_count += 1,
            Message::Soft => self.soft_count += 1,
        }

        self.update_input();
    }

    fn update_input(&mut self) {
        let Ok(mut pipe) = named_pipe::open_read(&self.input_path) else {
            return;
        }; // 非堵塞的打开管道，有新数据就更新

        let mut r = String::new();

        if pipe.read_to_string(&mut r).is_ok() {
            self.input = Self::parse_input(r);
            self.bound = Bound::new(self.input);
        }
    }

    fn connection_thread(mut pipe: File, rx: &Receiver<u32>) {
        loop {
            let level = rx.recv().unwrap();
            let _ = writeln!(pipe, "{level}");
        }
    }

    fn parse_input<S: AsRef<str>>(i: S) -> (u32, u32, Message) {
        let input = i.as_ref();

        let Some(input) = input.lines().last() else {
            error!("Failed to parse input, use default");
            return (120, 120, Message::Vsync);
        };

        let mut input = input.split(':');

        let target_fps = input
            .next()
            .and_then(|i| i.parse::<u32>().ok())
            .unwrap_or_else(|| {
                error!("Failed to parse target_fps");
                120
            });

        let display_fps = input
            .next()
            .and_then(|i| i.parse::<u32>().ok())
            .unwrap_or_else(|| {
                error!("Failed to parse display_fps");
                120
            });

        let message = input
            .next()
            .and_then(|i| match i {
                "vsync" => Some(Message::Vsync),
                "soft" => Some(Message::Soft),
                _ => None,
            })
            .unwrap_or_else(|| {
                error!("Failed to parse count_on");
                Message::Vsync
            });

        (target_fps, display_fps, message)
    }
}
