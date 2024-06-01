use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    #[error("Error operating on a file system")]
    FsError(#[from] io::Error),

    #[error("Error on serde json")]
    Serdejson(#[from] serde_json::Error),
}
