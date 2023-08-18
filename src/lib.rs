#![deny(clippy::all)]
#![warn(clippy::nursery)]

#[cfg(not(target_os = "android"))]
#[cfg(not(target_arch = "aarch64"))]
compile_error!("Only for aarch64 android");

mod error;
mod hook;

use std::{mem, ptr};

use android_logger::{self, Config};
use dobby_api::Address;
use log::{error, info, LevelFilter};

use error::Result;
use hook::SymbolHooker;

static mut VSYNC_FUNC_PTR: Address = ptr::null_mut();
static mut COMP_FUNC_PTR: Address = ptr::null_mut();
static mut GET_FPS_FUNC_PTR: Address = ptr::null_mut();

static mut COMPOSE_COUNT: usize = 0;
static mut VSYNC_COUNT: usize = 0;
static mut FPS: f32 = 0.0;
static mut PERIOD: i64 = 0;

#[no_mangle]
pub extern "C" fn handle_hook() {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("SURFACEFLINGER HOOK"),
    );

    unsafe {
        hook_main().unwrap_or_else(|e| error!("{e:#?}"));
    }
}

unsafe fn hook_main() -> Result<()> {
    let hooker = SymbolHooker::new()?;
    info!("Hooker started");

    let address = post_hook_vsync as Address;
    VSYNC_FUNC_PTR = hooker.find_and_hook(["DispSync", "onVsyncCallback"], address)?;

    info!("Hooked onVsyncCallback func");

    let address = post_hook_comp as Address;
    COMP_FUNC_PTR = hooker.find_and_hook(["SurfaceFlinger", "postComposition"], address)?;

    info!("Hooked postComposition func");

    Ok(())
}

// void onVsyncCallback(nsecs_t vsyncTime, nsecs_t targetWakeupTime, nsecs_t readyTime);
#[no_mangle]
unsafe extern "C" fn post_hook_vsync(a: i64, b: i64, c: i64) {
    let ori_func: extern "C" fn(i64, i64, i64) = mem::transmute(VSYNC_FUNC_PTR);
    ori_func(a, b, c);

    VSYNC_COUNT += 1;

    if VSYNC_COUNT > 1 {
        info!("{COMPOSE_COUNT}");

        COMPOSE_COUNT = 0;
        VSYNC_COUNT = 0;
    }
}

// void postComposition()
#[no_mangle]
unsafe extern "C" fn post_hook_comp() {
    let ori_func: extern "C" fn() = mem::transmute(COMP_FUNC_PTR);
    ori_func();

    COMPOSE_COUNT += 1;
}
