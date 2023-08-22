use std::io;

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Failed to parse lib: {source:?}")]
    LibParse {
        #[from]
        source: goblin::error::Error,
    },
    #[error("Failed to find target ymbol(s)")]
    Symbol,
    #[error("Dobby hook got an error: {source:?}")]
    DobbyError {
        #[from]
        source: dobby_api::Error,
    },
    #[error("Failed to creat named pipe")]
    NamedPipe,
    #[error("Got an io error: {source:?}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("An error happened: {0}")]
    Other(&'static str),
}
