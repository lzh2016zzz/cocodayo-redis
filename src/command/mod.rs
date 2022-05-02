use crate::command::get::Get;
use crate::command::set::Set;
use crate::command::select::Select;
use crate::command::info::Info;
use crate::command::del::Del;
use crate::command::mget::MGet;
use crate::command::exists::Exists;
use crate::command::flushdb::Flushdb;
use crate::protocol::frame::Frame;
use crate::protocol::parse::Parse;
use crate::server::shared::Shared;
use crate::command::strlen::StrLen;

use self::incrby::IncrBy;
use self::ttl::Ttl;


pub mod get;
pub mod set;
pub mod unknown;
pub mod select;
pub mod info;
pub mod del;
pub mod exists;
pub mod ttl;
pub mod flushdb;
pub mod incrby;
pub mod mget;
pub mod strlen;


#[derive(Debug)]
pub enum Command {
    GET(Get),
    SET(Set),
    DEL(Del),
    INCR(IncrBy),
    PING,
    FLUSHDB(Flushdb),
    EXISTS(Exists),
    INFO(Info),
    TTL(Ttl),
    PTTL(Ttl),
    SELECT(Select),
    MGET(MGet),
    UNKNOWN(String,Parse),
    STRLEN(StrLen)
}

impl Command {

    pub async fn apply(self,shared :&mut Shared) -> crate::Result<Frame>{
        
        let result = match self {
            Command::UNKNOWN(cmd,_) => Ok(Some(Frame::Error(format!("ERR unknown command '{}'",cmd)))),
            Command::PING => Ok(Some(Frame::Str(b"PONG".to_vec()))),
            Command::INFO(info) => info.apply(shared),
            Command::GET(get) => get.apply(shared),
            Command::SET(set) => set.apply(shared),
            Command::SELECT(select) => select.apply(shared),
            Command::DEL(del) => del.apply(shared),
            Command::EXISTS(_) => todo!(),
            Command::TTL(ttl) | Command::PTTL(ttl) => ttl.apply(shared),
            Command::FLUSHDB(flushdb) => flushdb.apply(shared),
            Command::INCR(incrby) => incrby.apply(shared),
            Command::MGET(mget) => mget.apply(shared),
            Command::STRLEN(strlen) => strlen.apply(shared),
        };

        return match result{
            Ok(frame) => Ok(frame.unwrap_or_else(||Frame::Nil)),
            Err(e) => Err(e.into()),
        }
            
    }
    
}

pub trait Execable {
    fn apply(self,shared :&mut Shared) -> crate::Result<Option<Frame>>;
}
