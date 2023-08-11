use std::{mem, time::Instant};

use log::debug;

use super::{utils, MMAP, ORI_FUN_ADDR, TIME_STAMP};

#[inline(never)]
#[no_mangle]
pub extern "C" fn post_composition_hooked() {
    unsafe {
        if !ORI_FUN_ADDR.is_null() {
            let ori_fun: fn() = mem::transmute(ORI_FUN_ADDR);
            ori_fun(); // 调用原函数
        }
    }

    let now = Instant::now();

    unsafe {
        if let Some(stamp) = TIME_STAMP {
            let frametime = Instant::now() - stamp;
            debug!("Frametime: {frametime:?}");

            if let Some(ref mut mmap) = MMAP {
                utils::update_mmap(mmap, frametime)
            }
        }

        TIME_STAMP = Some(now);
    }
}
