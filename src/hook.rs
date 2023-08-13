use std::{collections::VecDeque, mem, sync::mpsc::Receiver, time::Instant};

use log::error;

use crate::{utils::FileInterface, ORI_FUN_ADDR, SENDER};

const BUFFER_SIZE: usize = 1024;

#[inline(never)]
#[no_mangle]
pub extern "C" fn post_composition_hooked() {
    unsafe {
        let ori_fun: fn() = mem::transmute(ORI_FUN_ADDR);
        ori_fun(); // 调用原函数
        
        let now = Instant::now();
        if let Some(sx) = &SENDER {
            let _ = sx.0.send(now);
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
