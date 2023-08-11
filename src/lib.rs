mod hook;
pub(crate) mod utils;

use std::{
    ffi::{c_void, CString},
    ptr, str,
    time::Instant,
};

use android_logger::{self, Config};
use dobby_sys::ffi as dobby;
use log::{error, info, LevelFilter};
use memmap2::MmapMut;

pub(crate) const HOOK_DIR: &str = "/data/surfaceflinger_hook";

pub(crate) type Address = *mut c_void;

pub(crate) static mut TIME_STAMP: Option<Instant> = None;
pub(crate) static mut ORI_FUN_ADDR: Address = ptr::null_mut();
pub(crate) static mut MMAP: Option<MmapMut> = None;

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

    unsafe {
        MMAP = match utils::creat_mmap() {
            Ok(o) => {
                info!("Created mmap file");
                Some(o)
            }
            Err(e) => {
                error!("Fail to creat mmap file");
                error!("Reason: {e:?}");
                return;
            }
        };
    }

    let hook_address = hook::post_composition_hooked as Address;
    unsafe {
        dobby::DobbyHook(symbol, hook_address, &mut ORI_FUN_ADDR);
    }
}
