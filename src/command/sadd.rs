
use crate::{protocol::{parse::Parse, ParseError, frame::{self, Frame}, self}, server::value::Value};




#[derive(Debug)]
pub struct SAdd {
    key :Vec<u8>,
    values :Vec<Vec<u8>>
}

impl SAdd {
    pub fn parse(mut parse: Parse) -> Result<SAdd, ParseError> {
        
        let key =  match parse.next() {
            Ok(value) => value.into_vec()?,
            Err(e) => match e {
                ParseError::EOF => return Err("ERR wrong number of arguments for 'sadd' command".into()),
                err => return Err(err),
            },
        };

        let values = match parse.remaining_into_vec()  {
            Ok(value) => value,
            Err(e) => match e {
                ParseError::EOF => return Err("ERR wrong number of arguments for 'sadd' command".into()),
                err => return Err(err),
            },
        };

        Ok(SAdd{key,values})

    }
}

impl super::Execable for SAdd {
    fn apply(self,shared :&mut crate::server::shared::Shared) -> crate::Result<Option<frame::Frame>> {
        todo!()
    }
}