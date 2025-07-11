use chromiumoxide::error::CdpError;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug, Clone)]
pub enum BrowserError {
    #[error("failed to create page")]
    PageCreation,

    #[error("websocket communication failed")]
    WebSocket,

    #[error("connection timeout")]
    Timeout,

    #[error("network I/O error")]
    NetworkIO,

    #[error("browser launch failed")]
    BrowserLaunch,

    #[error("frame not found")]
    FrameNotFound,

    #[error("navigation failed")]
    Navigation,

    #[error("serialization error")]
    Serialization,

    #[error("decoding error")]
    Decoding,

    #[error("chrome internal error")]
    ChromeInternal,

    #[error("javascript exception")]
    JavaScriptError,

    #[error("invalid URL")]
    InvalidUrl,

    #[error("invalid browser config")]
    BuildBrowserConfigError,

    #[error("unknown error")]
    Unknown,
}

impl From<CdpError> for BrowserError {
    fn from(error: CdpError) -> Self {
        match error {
            CdpError::Ws(_) => BrowserError::WebSocket,
            CdpError::Io(_) => BrowserError::NetworkIO,
            CdpError::Serde(_) => BrowserError::Serialization,
            CdpError::Chrome(_) => BrowserError::ChromeInternal,
            CdpError::NoResponse => BrowserError::Timeout,
            CdpError::UnexpectedWsMessage(_) => BrowserError::WebSocket,
            CdpError::ChannelSendError(_) => BrowserError::NetworkIO,
            CdpError::LaunchExit(_, _) | CdpError::LaunchTimeout(_) | CdpError::LaunchIo(_, _) => {
                BrowserError::BrowserLaunch
            }
            CdpError::Timeout => BrowserError::Timeout,
            CdpError::FrameNotFound(_) => BrowserError::FrameNotFound,
            CdpError::ChromeMessage(_) => BrowserError::ChromeInternal,
            CdpError::DecodeError(_) => BrowserError::Decoding,
            CdpError::ScrollingFailed(_) => BrowserError::ChromeInternal,
            CdpError::NotFound => BrowserError::PageCreation,
            CdpError::JavascriptException(_) => BrowserError::JavaScriptError,
            CdpError::Url(_) => BrowserError::InvalidUrl,
        }
    }
}

impl From<tokio::time::error::Elapsed> for BrowserError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        BrowserError::Timeout
    }
}
