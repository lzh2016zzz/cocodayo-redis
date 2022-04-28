
use crate::protocol::{frame::{self}, parse::Parse, ParseError};

use super::Execable;

#[derive(Debug)]
pub struct Del {
    keys: Vec<String>,
}

impl Del {
    pub fn parse(parse: Parse) -> Result<Del, ParseError> {
        let keys = match parse.remaining_into_string_vec() {
            Ok(some) => some,
            Err(ParseError::EOF) => {
                return Err("ERR wrong number of arguments for 'del' command".into())
            }
            Err(e) => return Err(e.into()),
        };
        
        return Ok(Del{keys})
            
    }
}

impl Execable for Del {
    fn apply(self,shared :&mut crate::server::shared::Shared) -> crate::Result<Option<frame::Frame>> {
        let keys = self.keys;
        let mut num = 0;
        for key in keys.into_iter() {
            num += shared.del(&key);
        }

        Ok(Some(num.into()))
    }
}