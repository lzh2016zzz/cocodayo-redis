use core::fmt;
use std::error::Error;
use std::string::FromUtf8Error;



pub mod frame;
pub mod parse;


const NC: &[u8; 2] = b"\r\n";
const NIL: &[u8] = b"$-1\r\n";




#[derive(Debug)]
pub enum FrameError {
    Incomplete,
    Other(crate::Error),
}

impl Error for FrameError {}

impl From<&str> for FrameError {
    fn from(src: &str) -> FrameError {
        FrameError::Other(src.to_string().into())
    }
}

impl From<String> for FrameError {
    fn from(src: String) -> FrameError {
        FrameError::Other(src.into())
    }
}

impl fmt::Display for FrameError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrameError::Incomplete => "stream ended early".fmt(fmt),
            FrameError::Other(err) => err.fmt(fmt),
        }
    }
}


#[derive(Debug)]
pub enum ParseError {
    EOF,
    Other(crate::Error),
}

impl Error for ParseError {}

impl From<&str> for ParseError {
    fn from(src: &str) -> ParseError {
        ParseError::Other(src.to_string().into())
    }
}

impl From<FromUtf8Error> for ParseError {
    fn from(err: FromUtf8Error) -> Self {
        let err_msg = err.to_string();
        err_msg.into()
    }
}

impl From<String> for ParseError {
    fn from(src: String) -> ParseError {
        ParseError::Other(src.into())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::EOF => "end of stream".fmt(fmt),
            ParseError::Other(err) => err.fmt(fmt),
        }
    }
}

