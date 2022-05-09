


use crate::{
    protocol::{frame::Frame, parse::Parse, ParseError},
    server::value::Value,
};

use super::Execable;

#[derive(Debug)]
pub struct Set {
    key: String,
    value: Frame,
    expire_mill: Option<i64>,
    nx: Option<()>,
    xx: Option<()>,
}

impl Set {
    pub fn parse(p: Parse) -> Result<Set, ParseError> {
        let parse = |mut parse: Parse| {
            let key = parse.next()?.into_string()?;
            let value = parse.next()?;
            let mut expire_mill: Option<i64> = None;
            let mut nx: Option<()> = None;
            let mut xx: Option<()> = None;

            loop {
                match parse.next() {
                    Ok(frame) => {
                        let word = match frame {
                            Frame::Str(_) | Frame::Bulk(_) => frame.into_string(),
                            _ => return Err("ERR syntax error".into()),
                        };

                        if let Ok(kw) = word {
                            let keyword = &kw.to_lowercase()[..];
                            match keyword {
                                "nx" => nx = Some(()),
                                "xx" => xx = Some(()),
                                "ex" | "px" => {
                                    let next = match parse.next() {
                                        Ok(next) => next,
                                        Err(ParseError::EOF) => {
                                            return Err("ERR syntax error".into())
                                        }
                                        Err(e) => return Err(e.into()),
                                    };
                                    let expire: i64 = match next.into_decimal() {
                                        Ok(mut i) => {
                                            if keyword == "ex" {
                                                i *= 1000;
                                            }
                                            i
                                        }
                                        Err(_) => return Err("ERR syntax error".into()),
                                    };
                                    expire_mill = Some(expire)
                                }
                                _ => return Err("ERR syntax error".into()),
                            }
                        } else {
                            return Err("ERR syntax error".into());
                        }
                    }
                    Err(ParseError::EOF) => {
                        let set = Set {
                            key,
                            value,
                            expire_mill,
                            nx,
                            xx,
                        };
                        return Ok(set);
                    }
                    _ => return Err("ERR syntax error".into()),
                }
            }
        };

        let parse_rs = parse(p);

        match parse_rs {
            Ok(_) => return parse_rs,
            Err(ParseError::EOF) => {
                return Err("ERR wrong number of arguments for 'set' command".into())
            }
            Err(_) => return Err("ERR syntax error".into()),
        }
    }
}

impl Execable for Set {
    fn apply(
        self,
        shared: &mut crate::server::shared::Shared,
    ) -> crate::Result<Option<crate::protocol::frame::Frame>> {
        let value = self.value;

        let valueref = match value {
            Frame::Str(str) => Value::Bytes(str),
            Frame::Integer(_) => Value::Bytes(value.into_bytes()?.to_vec()),
            Frame::Bulk(b) => Value::Bytes(b[..].into()),
            _ => return Err("invalid data type".into()),
        };

        let _ = match self.expire_mill {
            Some(i@1..) => i,
            Some(_) => return Err("ERR invalid expire time in set".into()),
            _ => -1,
        };

        let set_result = shared.set(
            &self.key,
            valueref,
            self.nx.is_some()
        );

        match set_result {
            Ok(Some(_)) => Ok(Some(Frame::Str(b"OK".to_vec()))),
            Ok(None) => Ok(Some(Frame::Nil)),
            Err(err) => Err(err),
        }
    }
}
