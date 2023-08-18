use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Failed to parse /system/lib64/libsurfaceflinger.so")]
    LibParse,
    #[error("Failed to find target ymbol(s)")]
    Symbol,
    #[allow(unused)]
    #[error("An error happened: {0}")]
    Other(&'static str),
}
