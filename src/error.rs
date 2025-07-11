use crate::browser::BrowserError;
use crate::wbseller::error::WbSellerError;
use std::io::Error as StdIoError;
use std::result::Result as StdResult;
use thiserror::Error as ThisError;
use toml::de::Error as TomlDeError;

pub type Result<T> = StdResult<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("StdIoError: {0:?}")]
    StdIo(#[from] StdIoError),

    #[error("BrowserError: {0:?}")]
    Browser(#[from] BrowserError),

    #[error("WbSellerError: {0:?}")]
    WbSeller(#[from] WbSellerError),

    #[error("TomlDeError: {0:?}")]
    TomlDe(#[from] TomlDeError),
    // #[error("{0}")]
    // Custom(String),
}
