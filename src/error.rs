use std::io;
use std::result;
use std::string;

use crossbeam::channel::RecvError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed receiving message from channel: {0}")]
    ChannelReceive(#[from] RecvError),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("failed converting byte vector to UTF-8 String: {0}")]
    FromUtf8(#[from] string::FromUtf8Error),
}

pub type Result<T> = result::Result<T, Error>;
