use bytes::{Bytes};

use crate::protocol::{frame::Frame, parse::Parse, ParseError};

use crate::command::Execable;

#[derive(Debug)]
pub struct Info {
    section: Option<String>,
}

impl Info {
    pub fn parse(mut parse: Parse) -> Result<Info, ParseError> {
        let section = match parse.next() {
            Ok(Frame::Str(str)) => Some(str),
            Err(crate::protocol::ParseError::EOF) => None,
            Err(e) => return Err(e.into()),
            _ => None,
        };

        match parse.fin() {
            Ok(_) => {
                match section {
                    Some(s) =>  {
                        let section = match String::from_utf8(s) {
                            Ok(value) => value,
                            Err(_) => "".to_string(),
                        };
                        Ok(Info{section :Some(section)})
                    },
                    None => Ok(Info{ section:None}),
                }
            },
            Err(_) => return Err("ERR syntax error".into()),
        }
    }
}

impl Execable for Info {
    fn apply(self, shared: &mut crate::server::shared::Shared) -> crate::Result<Option<Frame>> {
        let mut buf = String::new();

        match self.section {
            Some(s) => match &s.to_lowercase()[..] {
                "all" | "keyspace" | "" => {
                    buf.push_str(
                        "# Keyspace
                        ",
                    );
                    let str = format!("db0:keys={},expires=0,avg_ttl=0", shared.len());
                    buf.push_str(&str[..]);
                    buf.push_str(
                        "
                        ",
                    );
                }
                _ => {}
            },
            None => {
                buf.push_str(
                    "# Keyspace
                    ",
                );
                let str = format!("db0:keys={},expires=0,avg_ttl=0", shared.len());
                buf.push_str(&str[..]);
                buf.push_str(
                    "
                    ",
                );
            }
        }
        Ok(Some(Frame::Bulk(buf.into_bytes())))
    }
}
