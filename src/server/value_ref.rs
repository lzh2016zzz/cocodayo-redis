
use std::{
    error::Error,
    fmt,
};

use bytes::{BufMut};

use crate::protocol::frame::Frame;

#[derive(Debug)]
pub enum ConvertError {
    InvalidNumberFormat,
    Other(crate::Error),
}

impl Error for ConvertError {}

impl From<&str> for ConvertError {
    fn from(src: &str) -> ConvertError {
        ConvertError::Other(src.to_string().into())
    }
}

impl From<String> for ConvertError {
    fn from(src: String) -> ConvertError {
        ConvertError::Other(src.into())
    }
}

impl fmt::Display for ConvertError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConvertError::InvalidNumberFormat => "invalid number format".fmt(fmt),
            ConvertError::Other(err) => err.fmt(fmt),
        }
    }
}

pub enum ValueRef {
    Bytes(Vec<u8>),
    None
}

impl ValueRef {
    pub fn frame(self) -> crate::Result<Frame> {
        match self {
            ValueRef::None => Ok(Frame::Nil),
            ValueRef::Bytes(u8) => Ok(Frame::Str(u8)),
        }
    }

    pub fn incr(&mut self, i: i64) -> Result<i64, ConvertError> {
        match self {
            ValueRef::Bytes(b) => {
                let fmt:Result<i64,ConvertError> = atoi::atoi::<i64>(b)
                    .ok_or_else(|| ConvertError::InvalidNumberFormat);

                match fmt {
                        Ok(mut num) => {
                            num += i;
                            b.clear();
                            b.put_slice(&format!("{}", num).into_bytes());
                            Ok(num)
                        },
                        Err(err) => return Err(err),
                }
            }
            _ => Err("WRONGTYPE Operation against a key holding the wrong kind of value".into()),
        }
    }

    pub fn as_slice(&self) -> &[u8]{
        match self {
            ValueRef::None => "".as_ref(),
            ValueRef::Bytes(r) => &r,
        }
    }

    pub fn from_u8(u8:Vec<u8>) -> Self {
        ValueRef::Bytes(u8)
    }
}
