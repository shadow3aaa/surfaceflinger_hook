use std::{
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write},
    path::Path,
    time::Duration,
};

use memmap2::MmapMut;

use super::HOOK_DIR;

// 目标symbol由shell调用readelf扫描，并且设置权限给surfaceflinger读取
pub fn target_symbol() -> Result<String, io::Error> {
    let path = Path::new(HOOK_DIR);
    fs::read_to_string(path.join("available_symbol"))
}

// 用于共享frametimes缓存(缓存512个帧间隔，单位ns)
// 由于mmap大小固定，每个frametime最大长度为100ms(100,000,000ns)，超过会截断
pub fn creat_mmap() -> Result<MmapMut, io::Error> {
    let path = Path::new(HOOK_DIR).join("frametimes");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    let mut init_writer = BufWriter::new(&file); // 因为写入量大，使用缓冲写入器
    for _ in 0..512 {
        writeln!(init_writer, "000000000")?; // 写512个长度为9的0
    }
    init_writer.flush()?;
    drop(init_writer);

    unsafe { MmapMut::map_mut(&file) }
}

// 更新frametime
pub fn update_mmap(m: &mut MmapMut, t: Duration) {
    let t = t.as_nanos().min(100000000); // 最大100ms
    let t = format!("{:09}\n", t); // 不足9位的，在前面填充0

    let mut up = m[10..].to_vec(); // 切掉10个u8，也就是一行(9个数字+一个\n)
    up.extend(t.as_bytes()); // 插入10个字符

    m.copy_from_slice(&up);
}
