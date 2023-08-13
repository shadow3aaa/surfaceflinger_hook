use std::{
    collections::VecDeque,
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write},
    path::Path,
    time::{Duration, Instant},
};

use memmap2::MmapMut;

use super::{FileInterface, HOOK_DIR};

const HOSTORY_LEN: usize = 512;

pub struct FrameTimesMmap {
    mmap: MmapMut,
}

impl FileInterface for FrameTimesMmap {
    fn init() -> Result<Self, io::Error> {
        let path = Path::new(HOOK_DIR).join("frametimes");
        let _ = fs::remove_file(&path); // 删除原来的节点

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        let mut writer = BufWriter::new(&file);
        for _ in 0..512 {
            writeln!(writer, "{:<9}", 0)?; // 初始化mmap为512 * 10字节
        }
        writer.flush()?;

        Ok(Self {
            mmap: unsafe { MmapMut::map_mut(&file)? },
        })
    }

    fn update(&mut self, b: &VecDeque<(Duration, Instant)>) -> Result<(), io::Error> {
        for (frametime, _) in b.iter().take(HOSTORY_LEN) {
            let frametime = format!("{:<9}\n", frametime.as_nanos()); // 左对齐，填充到9个字符，再加上\n 10个字符刚好不变

            let mut up = self.mmap[10..].to_vec(); // 切掉10个u8，也就是一行(9个数字+一个\n)
            up.extend(frametime.as_bytes()); // 插入10个字符(新frametime)到末尾
            self.mmap.copy_from_slice(&up); // 更新到mmap
        }

        Ok(())
    }
}
