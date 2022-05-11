use crate::{protocol::{frame::Frame, parse::Parse, ParseError}, server::shared::Shared};

use super::Execable;

#[derive(Debug)]
pub struct SMembers {
    key: String,
}

impl SMembers {
    pub fn parse(mut parse: Parse) -> Result<SMembers, ParseError> {
        let key = parse.next()?.into_string()?;
        match parse.fin() {
            Ok(_) => return Ok(SMembers { key }),
            Err(_) => return Err("ERR wrong number of arguments for 'smembers' command".into()),
        }
    }
}

impl Execable for SMembers {
    fn apply(self, shared: &mut Shared) -> crate::Result<Option<Frame>> {

        let key = self.key;
        let values = shared.sets_iterator(&key, -1)?;

        let mut frames = Vec::with_capacity(values.len());

        for val in values.into_iter() {
            let frame = val.frame()?;
            frames.push(frame);
        }
        Ok(Some(Frame::Array(frames)))
    }
}
