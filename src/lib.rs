/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

#[cfg(not(target_os = "android"))]
#[cfg(not(target_arch = "aarch64"))]
compile_error!("Only for aarch64 android");

mod analyze;
mod connect;
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

use error::Result;
use hook::SymbolHooker;

pub(crate) const API_DIR: &str = "/dev/surfaceflinger_hook";

static mut VSYNC_FUNC_PTR: Address = ptr::null_mut();
static mut SOFT_FUNC_PTR: Address = ptr::null_mut();

static mut VSYNC_SENDER: Option<Sender<Message>> = None;
static mut SOFT_SENDER: Option<Sender<Message>> = None;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Vsync,
    Soft,
}

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
    VSYNC_FUNC_PTR = hooker.find_and_hook(["DispSyncSource", "onVsyncCallback"], address)?;

    info!("Hooked onVsyncCallback func");

    let address = post_hook_comp as Address;
    SOFT_FUNC_PTR = hooker.find_and_hook(["SurfaceFlinger", "postComposition"], address)?;

    info!("Hooked commit func");

    let (sx, rx) = mpsc::channel();
    VSYNC_SENDER = Some(sx.clone());
    SOFT_SENDER = Some(sx);

    thread::Builder::new()
        .name("HookAnalyze".into())
        .spawn(move || analyze::jank(&rx))?;

    Ok(())
}

// void onVsyncCallback(nsecs_t vsyncTime, nsecs_t targetWakeupTime, nsecs_t readyTime);
#[no_mangle]
unsafe extern "C" fn post_hook_vsync(a: i64, b: i64, c: i64) {
    let ori_func: extern "C" fn(i64, i64, i64) -> () = mem::transmute(VSYNC_FUNC_PTR);
    ori_func(a, b, c);

    if let Some(sx) = &VSYNC_SENDER {
        sx.send(Message::Vsync).unwrap_or_else(|e| error!("{e:?}"));
    }
}

// void SurfaceFlinger::postComposition();
#[no_mangle]
unsafe extern "C" fn post_hook_comp() {
    let ori_func: extern "C" fn() -> () = mem::transmute(SOFT_FUNC_PTR);
    ori_func();

    if let Some(sx) = &SOFT_SENDER {
        sx.send(Message::Soft).unwrap_or_else(|e| error!("{e:?}"));
    }
}
