use crate::{protocol::{frame::{self, Frame}, parse::Parse, ParseError}, server::value_ref::Value};

use super::Execable;

#[derive(Debug)]
pub struct MSet {
    pairs: Vec<Vec<u8>>,
}

impl MSet {
    pub fn parse( mut parse: Parse) -> Result<MSet, ParseError> {
        let frames = parse.remaining()?;
        if frames.len() % 2 != 0 {
            return Err("ERR wrong number of arguments for 'mset'".into())
        }
        let mut pairs = Vec::with_capacity(frames.len());
        for item in frames.into_iter() {
            let vec = item.into_vec()?;
            pairs.push(vec);
        }
        return Ok(MSet{pairs})   
    }
}

impl Execable for MSet {
    fn apply(self,shared :&mut crate::server::shared::Shared) -> crate::Result<Option<frame::Frame>> {
        let mut pairs = self.pairs;
        while pairs.len() != 0 && pairs.len() % 2 == 0 {
            let value = Value::Bytes(pairs.pop().unwrap()); 
            let key =String::from_utf8(pairs.pop().unwrap())?;
            let _ = shared.set(&key, value, false);
        }
        return Ok(Some(Frame::Str(b"OK".to_vec())))
    }
}