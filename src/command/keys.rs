use crate::{
    protocol::{frame::Frame, parse::Parse, ParseError},
    server::shared::Shared,
};

use super::Execable;

#[derive(Debug)]
pub struct Keys {
    pattern: String,
}

impl Keys {
    pub fn parse(mut parse: Parse) -> Result<Keys, ParseError> {
        let pattern = parse.next()?.into_string()?;
        match parse.fin() {
            Ok(_) => return Ok(Keys { pattern }),
            Err(_) => return Err("ERR wrong number of arguments for 'KEYS' command".into()),
        }
    }
}

impl Execable for Keys {
    fn apply(self, shared: &mut Shared) -> crate::Result<Option<Frame>> {
        let (keys,_) = shared.scan_for(Some(&self.pattern), -1,0);
        let keys: Vec<Frame> = keys.into_iter().map(|key| key.frame().unwrap()).collect();
        Ok(Some(Frame::Array(keys)))
    }
}
