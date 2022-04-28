use crate::{
    protocol::{frame::Frame, parse::Parse, ParseError},
    server::{shared::Shared, value_ref::ValueRef},
};

use super::Execable;

#[derive(Debug)]
pub struct IncrBy {
    key: String,
    num: i64,
}

impl IncrBy {
    pub fn parse(mut parse: Parse,incr :bool) -> Result<IncrBy, ParseError> {
        let key = parse.next()?.into_string()?;

        let num = if incr {1} else {parse.next()?.into_decimal()?};

        match parse.fin() {
            Ok(_) => return Ok(IncrBy { key, num }),
            Err(_) => {
                return Err("ERR wrong number of arguments for 'incrby | incr' command".into())
            }
        }
    }
}

impl Execable for IncrBy {
    fn apply(self, shared: &mut Shared) -> crate::Result<Option<Frame>> {
        let key = self.key;
        let num = self.num;

        let optref = shared.get(&key);
        let value = match optref {
            Some(mut r) => {
                let increment =  r.incr(num)?;
                shared.set_default(&key, r)?;
                increment
            },
            None => {
                let value_ref = ValueRef::Mut("1".into());
                shared.set_default(&key, value_ref)?;
                1
            }
        };
        return Ok(Some(Frame::Integer(value)));
    }
}
