use crate::protocol::frame::{Frame, self};

use super::Executeable;


lazy_static! {
    pub static ref PONG: frame::Frame = Frame::Str("PONG".to_string());
}


pub struct Ping {
}

impl Executeable for Ping {
    fn apply(&self,_ :&mut crate::server::db::Shared) -> crate::Result<Option<crate::protocol::frame::Frame>> {
       Ok(Some(PONG))
    }
}