pub mod fps;
pub mod frametime;

use std::{
    collections::VecDeque,
    error::Error,
    ffi::CString,
    fs, io,
    path::Path,
    ptr,
    time::{Duration, Instant},
};

use dobby_sys::ffi as dobby;
use libc::c_void;

use super::HOOK_DIR;

pub trait FileInterface: Sync + Send {
    fn init() -> Result<Self, io::Error>
    where
        Self: Sized;

    fn update(&mut self, buffer: &VecDeque<(Duration, Instant)>) -> Result<(), io::Error>;

    fn boxed(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

// 目标symbol由shell调用readelf扫描，并且设置权限给surfaceflinger读取
// 需要扫描两个symbol: 开始合成和合成结束
pub unsafe fn target_symbol() -> Result<(*mut c_void, *mut c_void), Box<dyn Error>> {
    let pre_path = Path::new(HOOK_DIR).join("symbol_preComposition");
    let post_path = Path::new(HOOK_DIR).join("symbol_postComposition");

    let pre_symbol = fs::read_to_string(&pre_path)?;
    let post_symbol = fs::read_to_string(&post_path)?;

    let pre_symbol = CString::new(pre_symbol.trim())?;
    let post_symbol = CString::new(post_symbol.trim())?;

    let _ = fs::remove_file(&pre_path);
    let _ = fs::remove_file(&post_path); // 读取完即可删除

    let pre_symbol = dobby::DobbySymbolResolver(ptr::null(), pre_symbol.as_ptr());
    let post_symbol = dobby::DobbySymbolResolver(ptr::null(), post_symbol.as_ptr());

    Ok((pre_symbol, post_symbol))
}
