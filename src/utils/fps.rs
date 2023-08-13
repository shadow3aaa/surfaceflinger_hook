use std::{
    collections::VecDeque,
    fs::{self, OpenOptions},
    io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use log::error;
use memmap2::MmapMut;

use super::{FileInterface, HOOK_DIR};

pub struct FpsMmap {
    mmap: MmapMut,
    fps_sample_time: Duration,
    fps_sample_backend: (PathBuf, Instant),
}

impl FpsMmap {
    fn update_time(&mut self) -> Result<(), io::Error> {
        let (path, stamp) = &self.fps_sample_backend;

        // 1秒刷新一次
        if stamp.elapsed() > Duration::from_secs(1) {
            let time = fs::read_to_string(path)?;
            let time = time.trim().parse().unwrap_or(400);
            let time = time.clamp(0, 5000); // 最多采样5秒内的avg fps
            self.fps_sample_time = Duration::from_millis(time);

            fs::write(path, self.fps_sample_time.as_millis().to_string())?; // 写回结论值来同步
        }

        Ok(())
    }
}

impl FileInterface for FpsMmap {
    fn init() -> Result<Self, io::Error> {
        let path = Path::new(HOOK_DIR).join("fps");
        let _ = fs::remove_file(&path); // 删除原来的节点

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)?;

        file.metadata()?.permissions().set_mode(0o644);
        file.set_len(4)?; // 4字节可以表示0-9999fps，同时也4k对齐了

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        let path = Path::new(HOOK_DIR).join("fps_sample");
        let fps_sample_file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)?;

        fps_sample_file.metadata()?.permissions().set_mode(0o644);
        fs::write(&path, "400")?;

        Ok(Self {
            mmap,
            fps_sample_time: Duration::from_millis(400),
            fps_sample_backend: (path, Instant::now()),
        })
    }

    fn update(&mut self, b: &VecDeque<(Duration, Instant)>) -> Result<(), io::Error> {
        if let Err(e) = self.update_time() {
            error!("Error happened: {e:?}");
        }

        let frame_count = b
            .iter()
            .take_while(|(_, s)| s.elapsed() <= self.fps_sample_time)
            .count();

        let avg_fps = frame_count as f64
            * (Duration::from_secs(1).as_nanos() as f64 / self.fps_sample_time.as_nanos() as f64);
        let avg_fps = (avg_fps as usize).clamp(0, 9999);
        let avg_fps = format!("{avg_fps:<4}");

        self.mmap.copy_from_slice(avg_fps.as_bytes());

        Ok(())
    }
}
