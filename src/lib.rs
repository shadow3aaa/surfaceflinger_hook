mod hook;
mod utils;

use std::{
    ptr,
    time::Instant,
    ffi::{CString, c_void},
};

use android_logger::{self, Config};
use dobby_sys::ffi as dobby;
use log::{error, info, LevelFilter};

pub(crate) type Address = *mut c_void;

pub(crate) static mut TIME_STAMP: Option<Instant> = None;
pub(crate) static mut ORI_FUN_ADDR: Address = ptr::null_mut();

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

    let symbol = CString::new(symbol.trim()).unwrap();
    let symbol = unsafe {
        dobby::DobbySymbolResolver(ptr::null(), symbol.as_ptr())
    };

    if symbol.is_null() {
        error!("Target func not found");
        return;
    }

    let hook_address = hook::post_composition_hooked as Address;
    unsafe {
        dobby::DobbyHook(symbol, hook_address, &mut ORI_FUN_ADDR);
    }
}
