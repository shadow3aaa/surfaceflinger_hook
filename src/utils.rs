use std::{fs, io};

pub fn target_symbol() -> Result<String, io::Error> {
    fs::read_to_string("/cache/surfaceflinger_hook/available_symbol")
}
