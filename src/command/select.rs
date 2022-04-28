use crate::protocol::{frame::Frame, parse::Parse, ParseError};

use super::Execable;


#[derive(Debug)]
pub struct Select {
    index : i8
}

impl Select {

    pub fn parse ( mut parse :Parse) -> Result<Select, ParseError> {
        let index:i8 = match parse.next()?.into_decimal() {
            Ok(i64) => {
                 match i64.try_into() {
                    Ok(0) => 0,
                    _ => return Err("ERR DB index is out of range".into()),
                }
            },
            Err(_) => return Err("ERR invalid DB index".into()),
        };

        match parse.fin() {
            Ok(_) => return Ok(Select { index }),
            Err(_) => return Err("ERR wrong number of arguments for 'select' command".into()),
        }
    }
}


impl Execable for Select {
    fn apply(self,_ :&mut crate::server::shared::Shared) -> crate::Result<Option<Frame>> {
        Ok(Some(Frame::Str(b"OK".to_vec())))
    }
}