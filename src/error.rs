#![allow(missing_docs)]

use std::{
    borrow::Cow, //
    error,
    fmt,
};

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(ErrorKind);

#[derive(Debug)]
enum ErrorKind {
    Msg(Cow<'static, str>),
}

impl From<&'static str> for Error {
    fn from(msg: &'static str) -> Self {
        Error(ErrorKind::Msg(Cow::Borrowed(msg)))
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error(ErrorKind::Msg(Cow::Owned(msg)))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ErrorKind::Msg(ref msg) => f.write_str(&*msg),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

macro_rules! bail {
    ($msg:expr) => {
        return Err($crate::error::Error::from($msg));
    };
}
