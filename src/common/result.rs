use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    BencodeDecode(String),
    Request(String),
    SerdeCustom(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Request(format!("{:?}", err))
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BencodeDecode(str) => {
                write!(f, "Decode error: {}", str)
            }
            Error::Request(str) => {
                write!(f, "Request error: {}", str)
            }
            Error::SerdeCustom(str) => {
                write!(f, "Serde custom error: {}", str)
            }
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::SerdeCustom(msg.to_string())
    }
}
