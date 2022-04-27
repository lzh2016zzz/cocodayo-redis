use crate::protocol::{
    frame::{self},
    parse::Parse,
    ParseError,
};

use super::Execable;

#[derive(Debug)]
pub struct Ttl {
    key: String,
    px: bool,
}

impl Ttl {
    pub fn parse(mut parse: Parse,pttl:bool) -> Result<Ttl, ParseError> {
        let key = parse.next()?.into_string()?;
        match parse.fin() {
            Ok(_) => return Ok(Ttl { key, px:pttl }),
            Err(_) => {
                if pttl {
                    return Err("ERR wrong number of arguments for 'pttl' command".into())
                }else {
                    return Err("ERR wrong number of arguments for 'ttl' command".into())
                }
            }
        }
    }
}

impl Execable for Ttl {
    fn apply(
        self,
        _: &mut crate::server::shared::Shared,
    ) -> crate::Result<Option<frame::Frame>> {
        Ok(Some((-1).into()))
    }
}
