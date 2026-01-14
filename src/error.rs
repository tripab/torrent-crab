use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Bencode decode error: {0}")]
    BencodeDecode(String),

    #[error("Bencode encode error: {0}")]
    BencodeEncode(String),

    #[error("Invalid metainfo: {0}")]
    InvalidMetainfo(String),

    #[error("Tracker error: {0}")]
    Tracker(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub type Result<T> = std::result::Result<T, Error>;
