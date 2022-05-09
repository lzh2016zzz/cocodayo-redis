use bytes::{Buf, BufMut, BytesMut};
use std::io::Cursor;

use crate::protocol::{FrameError, ParseError, NC, NIL};

#[derive(Clone, Debug)]
pub enum Frame {
    Str(Vec<u8>),
    Error(String),
    Integer(i64),
    Bulk(Vec<u8>),
    Array(Vec<Frame>),
    Nil,
}


impl Frame {


    pub fn len(&self) -> usize {
        match self {
            Frame::Str(s) => s.len(),
            Frame::Error(err) => err.len(),
            Frame::Integer(i) => format!("{}",i).len(),
            Frame::Bulk(b) => b.len(),
            Frame::Array(arr) => {
                let mut len:usize = 0;
                for item in arr {
                    len += item.len()
                }
                len
            },
            Frame::Nil => 0,
        }
    }


    pub fn into_vec(self) -> Result<Vec<u8>, ParseError>{
        match self {
            Frame::Bulk(bytes) | Frame::Str(bytes) => Ok(bytes),
            frame => Err(format!(
                "protocol error; expected simple frame or bulk frame, got {:?}",
                frame
            )
            .into()),
        }
    }

    pub fn into_string(self) -> Result<String, ParseError> {
        match self {
            Frame::Bulk(bytes) | Frame::Str(bytes) => {
                let b = bytes.to_vec();
                let str = match String::from_utf8(b) {
                    Ok(s) => s,
                    Err(e) => return Err(e.into()),
                };
                Ok(str)
            }
            frame => Err(format!(
                "protocol error; expected simple frame or bulk frame, got {:?}",
                frame
            )
            .into()),
        }
    }

    pub fn into_decimal(self) -> Result<i64, ParseError> {
        match self {
            Frame::Integer(i) => Ok(i),
            Frame::Str(s) => atoi::atoi::<i64>(s.as_slice())
                .ok_or_else(|| "protocol error invalid number format".into()),
            Frame::Bulk(s) => atoi::atoi::<i64>(s.as_slice())
                .ok_or_else(|| "protocol error invalid number format".into()),
            _ => Err("invalid protocol".into()),
        }
    }

    pub fn into_bytes(self) -> Result<BytesMut, ParseError> {
        return match self {
            Frame::Str(str) => {
                let capacity = str.len() + 10;
                let mut buf: BytesMut = BytesMut::with_capacity(capacity);
                buf.put_u8(b'+');
                buf.put_slice(&str);
                buf.put_slice(NC);
                Ok(buf)
            }
            Frame::Error(str) => {
                let mut buf: BytesMut = BytesMut::new();
                buf.put_u8(b'-');
                buf.put_slice(&str.into_bytes());
                buf.put_slice(NC);
                Ok(buf)
            }
            Frame::Integer(i) => {
                let mut buf: BytesMut = BytesMut::new();
                buf.put_u8(b':');
                buf.put_slice(&i_to_string(i as usize).into_bytes());
                buf.put_slice(NC);
                Ok(buf)
            }
            Frame::Bulk(slice) => {
                let s = &i_to_string(slice.len() as usize).into_bytes();
                let mut buf: BytesMut = BytesMut::with_capacity(slice.len() + 10);
                buf.put_u8(b'$');
                buf.put_slice(s);
                buf.put_slice(NC);
                buf.put(&slice[..]);
                buf.put_slice(NC);
                Ok(buf)
            }
            Frame::Array(arr) => {
                let mut buf: BytesMut = BytesMut::new();
                buf.put_u8(b'*');
                buf.put_slice(i_to_string(arr.len()).as_bytes());
                buf.put_slice(NC);
                for sub in arr {
                    let b = sub.into_bytes()?;
                    buf.put_slice(&b[..]);
                    drop(b);
                }
                Ok(buf)
            }
            Frame::Nil => Ok(BytesMut::from(NIL)),
        };
    }
}

pub fn parse_frame(src: &mut Cursor<&[u8]>) -> Result<Frame, FrameError> {
    if !src.has_remaining() {
        return Err(FrameError::Incomplete);
    }

    let s = get_u8(src)?;
    match s {
        b'+' => {
            let str = read_string(src)?;
            return Ok(Frame::Str(str.into_bytes()));
        }

        b'-' => {
            let str = read_string(src)?;
            return Ok(Frame::Error(str));
        }

        b':' => {
            let decimal = read_decimal(src)? as i64;
            Ok(Frame::Integer(decimal))
        }

        b'$' => {
            if peek_u8(src)? == b'-' {
                return if read_line(src)? != b"-1" {
                    Err("invalid protocol,got - ".into())
                } else {
                    Ok(Frame::Nil)
                };
            } else {
                let len = read_decimal(src)? as usize;
                let bulk = read_bulk(src, len)?;
                Ok(Frame::Bulk(bulk))
            }
        }

        b'*' => {
            let lines = read_decimal(src)?;
            let mut vec = vec![];
            for _ in 0..lines {
                vec.push(parse_frame(src)?)
            }
            Ok(Frame::Array(vec))
        }

        u8 => Err(format!("invalid protocol , got {}", u8).into()),
    }
}

pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), FrameError> {
    if !src.has_remaining() {
        return Err(FrameError::Incomplete);
    }

    let s = get_u8(src)?;
    match s {
        b'+' => {
            read_line(src)?;
            Ok(())
        }

        b'-' => {
            read_line(src)?;
            Ok(())
        }

        b':' => {
            read_decimal(src)?;
            Ok(())
        }

        b'$' => {
            if peek_u8(src)? == b'-' {
                if read_line(src)? != b"-1" {
                    return Err("invalid protocol ".into());
                }
                skip(src, b"-1\r\n".len())
            } else {
                let len = read_decimal(src)? as usize;
                skip(src, len + (NC.len()))
            }
        }

        b'*' => {
            let lines = read_decimal(src)?;

            for _ in 0..lines {
                check(src)?
            }
            Ok(())
        }

        u8 => Err(format!("invalid protocol , got {}", u8).into()),
    }
}

impl From<&str> for Frame {
    fn from(src: &str) -> Frame {
        Frame::Str(src.as_bytes().to_vec())
    }
}
impl From<i8> for Frame {
    fn from(src: i8) -> Frame {
        Frame::Integer(src.into())
    }
}
impl From<i16> for Frame {
    fn from(src: i16) -> Frame {
        Frame::Integer(src.into())
    }
}
impl From<i32> for Frame {
    fn from(src: i32) -> Frame {
        Frame::Integer(src.into())
    }
}
impl From<i64> for Frame {
    fn from(src: i64) -> Frame {
        Frame::Integer(src)
    }
}
impl From<usize> for Frame {
    fn from(src: usize) -> Frame {
        if let Ok(i) = src.try_into(){
            return Frame::Integer(i);
        }else {
            Frame::Error("numeric out of range".into())
        }
    }
}

impl From<Vec<u8>> for Frame {
    fn from(src: Vec<u8>) -> Self {
        match String::from_utf8(src) {
            Ok(s) => Frame::Str(s.into_bytes()),
            Err(_) => return Frame::Str("".into()),
        }
    }
}

impl From<Vec<Frame>> for Frame {
    fn from(src: Vec<Frame>) -> Self {
        Frame::Array(src)
    }
}

fn read_string(src: &mut Cursor<&[u8]>) -> Result<String, FrameError> {
    let line = read_line(src)?;
    let string = match String::from_utf8(line.to_vec()) {
        Ok(str) => str,
        Err(err) => return Err(FrameError::Other(err.into())),
    };
    return Ok(string);
}

fn read_bulk(src: &mut Cursor<&[u8]>, len: usize) -> Result<Vec<u8>, FrameError> {
    if src.remaining() < len {
        return Err(FrameError::Incomplete);
    }

    let b = &src.chunk()[..len];

    let data = b.to_vec();

    skip(src, len + b"\r\n".len())?;

    return Ok(data);
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !src.has_remaining() {
        return Err(FrameError::Incomplete);
    }
    return Ok(src.get_u8());
}

fn read_line<'a>(src: &'a mut Cursor<&[u8]>) -> Result<&'a [u8], FrameError> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(FrameError::Incomplete)
}

fn read_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, FrameError> {
    let line = read_line(src)?;

    atoi::atoi::<u64>(line).ok_or_else(|| "protocol error invalid number format".into())
}

fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !src.has_remaining() {
        return Err(FrameError::Incomplete);
    }

    Ok(src.chunk()[0])
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), FrameError> {
    if src.remaining() < n {
        return Err(FrameError::Incomplete);
    }
    src.advance(n);
    Ok(())
}

fn i_to_string(n: usize) -> String {
    return n.to_string();
}

