use crate::{protocol::{frame::Frame, parse::Parse, ParseError}, server::shared::Shared};

use super::Execable;

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl Get {
    pub fn parse(mut parse: Parse) -> Result<Get, ParseError> {
        let key = parse.next()?.into_string()?;
        match parse.fin() {
            Ok(_) => return Ok(Get { key }),
            Err(_) => return Err("ERR wrong number of arguments for 'get' command".into()),
        }
    }
}

impl Execable for Get {
    fn apply(self, shared: &mut Shared) -> crate::Result<Option<Frame>> {
        let key = &self.key;
        let value_ref = shared.get(key);
        if let Some(value) = value_ref {
            let frame = value.frame()?;
            let fmt = match frame {
                Frame::Str(_) | Frame::Nil => frame,
                Frame::Bulk(b) => Frame::Str(String::from_utf8(b.to_vec())?),
                Frame::Integer(i) => Frame::Str(format!("{}", i)),
                _ => {
                    return Err(
                        "WRONGTYPE Operation against a key holding the wrong kind of value".into(),
                    )
                }
            };
            return Ok(Some(fmt));
        }

        return Ok(Some(Frame::Nil));
    }
}
