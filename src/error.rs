use std::result;

use crossbeam::channel::RecvError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed receiving message from channel: {0}")]
    ChannelReceive(#[from] RecvError),
}

pub type Result<T> = result::Result<T, Error>;
