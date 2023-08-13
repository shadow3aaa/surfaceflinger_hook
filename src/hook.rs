use std::{
    collections::VecDeque,
    mem,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use libc::c_void;
use log::{error, info};

use crate::{utils::FileInterface, ORI_POST_COMP_ADDR, ORI_PRE_COMP_ADDR, SENDER};

static mut PRE_STAMP: Option<Instant> = None;

const BUFFER_SIZE: usize = 1024;

// void preComposition(CompositionRefreshArgs&)
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn pre_composition_hooked(args: c_void) {
    let ori_fun: extern "C" fn(c_void) = mem::transmute(ORI_PRE_COMP_ADDR);
    ori_fun(args);

    PRE_STAMP = Some(Instant::now());
}

// void SurfaceFlinger::postComposition()
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn post_composition_hooked() {
    let ori_fun: fn() = mem::transmute(ORI_POST_COMP_ADDR);
    ori_fun(); // 调用原函数

    if let Some(stamp) = PRE_STAMP {
        let now = Instant::now();
        let frametime = now - stamp;

        info!("{frametime:?}");

        if let Some(sx) = &SENDER {
            let _ = sx.0.send((frametime, now));
        }
    }
}

pub fn hook_thread(rx: &Receiver<(Duration, Instant)>, mut itf: Vec<Box<dyn FileInterface>>) {
    let mut buffer = VecDeque::with_capacity(BUFFER_SIZE);
    loop {
        let stamp = rx.recv().unwrap();
        buffer.push_front(stamp);

        if buffer.len() >= BUFFER_SIZE {
            buffer.pop_back();
        } // 保持buffer大小

        for i in &mut itf {
            if let Err(e) = i.update(&buffer) {
                error!("Error happened: {e:?}");
            }
        }
    }
}
