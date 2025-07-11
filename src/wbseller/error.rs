use crate::browser::BrowserError;
use reqwest::Error as ReqwestError;
use std::result::Result as StdResult;
use thiserror::Error;

pub type Result<T> = StdResult<T, WbSellerError>;

#[derive(Error, Debug)]
pub enum WbSellerError {
    #[error("BrowserError: {0:?}")]
    Browser(#[from] BrowserError),

    #[error("ReqwestError: {0:?}")]
    Reqwest(#[from] ReqwestError),

    #[error("{0}")]
    Custom(String),
}
