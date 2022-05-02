use crate::protocol::{parse::Parse, ParseError, frame::{self, Frame}};

use super::Execable;

#[derive(Debug)]
pub struct StrLen {
    key: String,
}

impl StrLen {
    pub fn parse( mut parse: Parse) -> Result<StrLen, ParseError> {
        let keys = match parse.next()?.into_string() {
            Ok(some) => some,
            Err(ParseError::EOF) => {
                return Err("ERR wrong number of arguments for 'strLen' command".into())
            }
            Err(e) => return Err(e.into()),
        };
        
        return Ok(StrLen{key: keys})
    }
}

impl Execable for StrLen {
    fn apply(self,shared :&mut crate::server::shared::Shared) -> crate::Result<Option<frame::Frame>> {
        let key = &self.key;
        let value_ref = shared.get(key);
        if let Some(value) = value_ref {
            let frame = value.frame()?;
            return Ok(Some(Frame::Str(format!("{}",frame.len()).into())));
        }

        return Ok(Some(Frame::Str("0".into())))

    }
}