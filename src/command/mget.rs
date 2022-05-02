
use crate::{protocol::{parse::Parse, ParseError, frame::{self, Frame}}, server::value_ref::{ValueRef}};




#[derive(Debug)]
pub struct MGet {
    keys :Vec<String>
}

impl MGet {
    pub fn parse(parse: Parse) -> Result<MGet, ParseError> {
        
        let keys = match parse.remaining_into_string_vec() {
            Ok(some) => some,
            Err(ParseError::EOF) => {
                return Err("ERR wrong number of arguments for 'mget' command".into())
            }
            Err(e) => return Err(e.into()),
        };
        
        return Ok(MGet{keys})
            
    }
}

impl super::Execable for MGet {
    fn apply(self,shared :&mut crate::server::shared::Shared) -> crate::Result<Option<frame::Frame>> {
        let keys = self.keys;
        let mut result = Vec::with_capacity(keys.len());
        for key in keys {
            match shared.get(&key) {
                Some(ValueRef::Bytes(r)) => {
                    result.push(r.into());
                },
                Some(ValueRef::None) | None => result.push(Frame::Nil),
            }
        }
        Ok(Some(result.into()))
    }
}