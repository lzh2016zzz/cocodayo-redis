

use crate::protocol::{ParseError, parse::Parse};

use super::Execable;


#[derive(Debug)]
pub struct Exists {
    keys: Vec<String>,
}


impl Exists {

    pub fn parse(parse: Parse) -> Result<Exists, ParseError> {

        let keys = match parse.remaining_into_string_vec() {
            Ok(some) => some,
            Err(ParseError::EOF) => {
                return Err("ERR wrong number of arguments for 'exists' command".into())
            }
            Err(e) => return Err(e.into()),
        };
        
        return Ok(Exists{keys})
            
    }
}

impl Execable for Exists {
    fn apply(self,shared :&mut crate::server::shared::Shared) -> crate::Result<Option<crate::protocol::frame::Frame>> {
        let keys = self.keys;
        let cnt = keys.into_iter().map(|key|shared.is_exists(&key)).filter(|&s|s).count();
        let frame = cnt.into();
        return Ok(Some(frame));
    }
}