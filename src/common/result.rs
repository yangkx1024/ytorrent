#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum Error {
    BencodeDecode(String),
    BencodeSerde(String),
    Request(String),
}

#[allow(dead_code)]
pub(crate) type Result<T> = std::result::Result<T, Error>;

impl From<bendy::decoding::Error> for Error {
    fn from(err: bendy::decoding::Error) -> Self {
        Error::BencodeDecode(format!("{:?}", err))
    }
}

impl From<bendy::serde::Error> for Error {
    fn from(err: bendy::serde::Error) -> Self {
        Error::BencodeSerde(format!("{:?}", err))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Request(format!("{:?}", err))
    }
}

