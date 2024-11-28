use std::fmt;

use tokio::task::JoinError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum SystemError {
    #[allow(unused, reason = "Example error")]
    Generic(String),

    #[error(transparent)]
    Join(#[from] JoinError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    RetryError,

    RequestError(String),

    Selector(String),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}

impl fmt::Display for SystemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic(msg) => write!(formatter, "GenericError: {msg}"),
            Self::Join(msg) => write!(formatter, "JoinError: {msg}"),
            Self::Sqlx(msg) => write!(formatter, "SqlxError: {msg}"),
            Self::Reqwest(msg) => write!(formatter, "ReqwestError: {msg}"),
            Self::RetryError => write!(formatter, "ReqwestError"),
            Self::RequestError(msg) => write!(formatter, "RequestError; {msg}"),
            Self::Selector(msg) => write!(formatter, "SelectorError: {msg}"),
            Self::UrlParse(msg) => write!(formatter, "UrlParseError: {msg}"),
        }
    }
}

pub(crate) type SystemResult<T, E = SystemError> = Result<T, E>;
