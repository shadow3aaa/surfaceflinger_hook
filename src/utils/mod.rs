pub mod fps;
pub mod frametime;

use std::{collections::VecDeque, fs, io, path::Path, time::Instant};

use super::HOOK_DIR;

pub trait FileInterface: Sync + Send {
    fn init() -> Result<Self, io::Error>
    where
        Self: Sized;

    fn update(&mut self, buffer: &VecDeque<Instant>) -> Result<(), io::Error>;

    fn boxed(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

// 目标symbol由shell调用readelf扫描，并且设置权限给surfaceflinger读取
pub fn target_symbol() -> Result<String, io::Error> {
    let path = Path::new(HOOK_DIR).join("available_symbol");
    let result = fs::read_to_string(&path)?;
    let _ = fs::remove_file(&path); // 读取完此文件就没意义了
    Ok(result)
}
