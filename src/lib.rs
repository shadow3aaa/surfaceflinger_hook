#![deny(clippy::all)]
#![warn(clippy::nursery)]
mod hook;
pub(crate) mod utils;

use std::{
    ffi::{c_void, CString},
    marker::Sync,
    ptr, str,
    sync::mpsc::{self, Sender},
    thread,
    time::Instant,
};

use android_logger::{self, Config};
use dobby_sys::ffi as dobby;
use log::{error, info, LevelFilter};

use utils::{fps::FpsMmap, frametime::FrameTimesMmap, FileInterface};

pub(crate) const HOOK_DIR: &str = "/dev/surfaceflinger_hook";

pub(crate) type Address = *mut c_void;

pub(crate) struct StampSender(Sender<Instant>);

unsafe impl Sync for StampSender {} // 实际上这不安全，但是hook不能通过参数传递它，只能通过static变量

pub(crate) static mut ORI_FUN_ADDR: Address = ptr::null_mut();
pub(crate) static mut SENDER: Option<StampSender> = None;

// no_mangle保证symbol不被修改
#[no_mangle]
pub extern "C" fn hook_surfaceflinger() {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("SURFACEFLINGER HOOK"),
    );

    info!("Start to hook");
    let symbol = match utils::target_symbol() {
        Ok(o) => {
            if o.is_empty() {
                error!("Target symbol not found");
                return;
            }
            info!("Try hook symbol {o}");
            o
        }
        Err(e) => {
            error!("Can not read target symbol file");
            error!("Reason: {e:?}");
            return;
        }
    };

    let symbol = CString::new(symbol.trim()).unwrap(); // 转为c兼容字符串
    let symbol = unsafe { dobby::DobbySymbolResolver(ptr::null(), symbol.as_ptr()) };

    if symbol.is_null() {
        error!("Target func not found");
        return;
    }

    let frametimes_node = match FrameTimesMmap::init() {
        Ok(o) => o.boxed(),
        Err(e) => {
            error!("Failed to init frametime node");
            error!("Reason: {e:?}");
            return;
        }
    };

    let fps_node = match FpsMmap::init() {
        Ok(o) => o.boxed(),
        Err(e) => {
            error!("Failed to init fps node");
            error!("Reason: {e:?}");
            return;
        }
    };

    let itfs: Vec<Box<dyn FileInterface>> = vec![frametimes_node, fps_node];

    let (sx, rx) = mpsc::channel();
    unsafe {
        SENDER = Some(StampSender(sx));
    }

    if let Err(e) = thread::Builder::new()
        .name("HookThread".into())
        .spawn(move || hook::hook_thread(&rx, itfs))
    {
        error!("Fail to creat hook thread");
        error!("Reason: {e:?}");
        return;
    }

    let hook_address = hook::post_composition_hooked as Address;
    unsafe {
        dobby::DobbyHook(symbol, hook_address, &mut ORI_FUN_ADDR);
    }
}
