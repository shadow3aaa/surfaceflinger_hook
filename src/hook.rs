use std::{collections::VecDeque, mem, sync::mpsc::Receiver, time::Instant};

use libc::int64_t;
use log::{error, info};

use crate::{utils::FileInterface, ORI_COMPOSITE_ADDR, ORI_COMPOSITION_ADDR, SENDER};

const BUFFER_SIZE: usize = 1024;

// void SurfaceFlinger::postComposition()
// or
// void SurfaceFlinger::postFrame()
#[inline(never)]
#[no_mangle]
pub extern "C" fn post_composition_hooked() {
    unsafe {
        let ori_fun: fn() = mem::transmute(ORI_COMPOSITION_ADDR);
        ori_fun(); // 调用原函数

        let now = Instant::now();
        if let Some(sx) = &SENDER {
            let _ = sx.0.send(now);
        }
    }
}

static mut temp_lock: Option<i64> = None;
// void SurfaceFlinger::composite(nsecs_t frameTime, int64_t vsyncId)
#[inline(never)]
#[no_mangle]
pub extern "C" fn post_composite_hooked(frametime: i64, vsync_id: i64) {
    unsafe {
        let ori_fun: extern "C" fn(i64, i64) = mem::transmute(ORI_COMPOSITE_ADDR);
        ori_fun(frametime, vsync_id);

        unsafe {
            if let Some(ref mut stamp) = temp_lock {
                info!("{:?}", vsync_id - *stamp);
                *stamp = vsync_id;
            } else {
                temp_lock = Some(vsync_id);
            }
        }
    }
}

pub fn hook_thread(rx: &Receiver<Instant>, mut itf: Vec<Box<dyn FileInterface>>) {
    let mut buffer = VecDeque::with_capacity(BUFFER_SIZE);
    loop {
        let stamp = rx.recv().unwrap();
        buffer.push_back(stamp);

        if buffer.len() >= BUFFER_SIZE {
            buffer.pop_front();
        } // 保持buffer大小

        for i in &mut itf {
            if let Err(e) = i.update(&buffer) {
                error!("Error happened: {e:?}");
            }
        }
    }
}
