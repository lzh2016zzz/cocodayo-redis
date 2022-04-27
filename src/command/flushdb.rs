
use crate::{protocol::{parse::Parse, ParseError, frame::Frame}, server::shared::Shared};

use super::Execable;

#[derive(Debug)]
pub struct Flushdb {}

impl Flushdb {
    pub fn parse(mut parse: Parse) -> Result<Flushdb, ParseError> {
        match parse.fin() {
            Ok(_) => return Ok(Flushdb { }),
            Err(_) => return Err("ERR wrong number of arguments for 'flushdb' command".into()),
        }
    }
}

impl Execable for Flushdb {
    fn apply(self, shared: &mut Shared) -> crate::Result<Option<Frame>> {
        shared.flush();
        Ok(Some("OK".into()))
    }
}
