#![deny(clippy::all)]
#![warn(clippy::nursery)]

#[cfg(not(target_os = "android"))]
#[cfg(not(target_arch = "aarch64"))]
compile_error!("Only for aarch64 android");

mod analyze;
mod error;
mod hook;

use std::{
    mem, ptr,
    sync::mpsc::{self, Sender},
    thread,
};

use android_logger::{self, Config};
use dobby_api::Address;
use log::{error, info, LevelFilter};

use analyze::Message;
use error::Result;
use hook::SymbolHooker;

static mut VSYNC_FUNC_PTR: Address = ptr::null_mut();
static mut COMM_FUNC_PTR: Address = ptr::null_mut();

static mut VSYNC_SENDER: Option<Sender<Message>> = None;
static mut COMM_SENDER: Option<Sender<Message>> = None;

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

    let address = post_hook_commit as Address;
    COMM_FUNC_PTR = hooker.find_and_hook(["SurfaceFlinger", "commit"], address)?;

    info!("Hooked commit func");

    let (sx, rx) = mpsc::channel();
    COMM_SENDER = Some(sx.clone());
    COMM_SENDER = Some(sx);

    thread::Builder::new()
        .name("HookThread".into())
        .spawn(move || analyze::jank(&rx))?;

    Ok(())
}

// void onVsyncCallback(nsecs_t vsyncTime, nsecs_t targetWakeupTime, nsecs_t readyTime);
#[no_mangle]
unsafe extern "C" fn post_hook_vsync(a: i64, b: i64, c: i64) {
    let ori_func: extern "C" fn(i64, i64, i64) = mem::transmute(VSYNC_FUNC_PTR);
    ori_func(a, b, c);

    if let Some(sx) = &VSYNC_SENDER {
        sx.send(Message::Vsync).unwrap_or_else(|e| error!("{e:?}"));
    }
}

// bool SurfaceFlinger::commit(nsecs_t frameTime, int64_t vsyncId, nsecs_t expectedVsyncTime)
#[no_mangle]
unsafe extern "C" fn post_hook_commit(a: i64, b: i64, c: i64) -> bool {
    let ori_func: extern "C" fn(i64, i64, i64) -> bool = mem::transmute(COMM_FUNC_PTR);
    let result = ori_func(a, b, c);

    if let Some(sx) = &COMM_SENDER {
        if result {
            sx.send(Message::Soft).unwrap_or_else(|e| error!("{e:?}"));
        }
    }

    result
}
