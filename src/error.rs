use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unknown {0} version")]
    UnknownVersion(String),
    #[error("{0} not found")]
    NotFound(String),
    #[error("Could not parse: {0}")]
    Parse(String),
    #[error("Could not take optional value: {0}")]
    Take(String),
    #[error("Download failed with status code: {0}")]
    Download(String),
    #[error("Timeout error")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("{0}")]
    Authentication(String),
    #[error("Malformed token: {0}")]
    MalformedToken(String),
    #[error("Operation failed: {0}")]
    Fail(String),
    #[error("Unsupported architecture")]
    UnsupportedArchitecture,
    #[error(transparent)]
    IO(#[from] tokio::io::Error),
    #[error(transparent)]
    StripPrefix(#[from] std::path::StripPrefixError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    FromUTF8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    OAuthUrlParse(#[from] oauth2::url::ParseError),
}
