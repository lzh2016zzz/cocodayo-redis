use crate::command::del::Del;
use crate::command::exists::Exists;
use crate::command::flushdb::Flushdb;
use crate::command::get::Get;
use crate::command::incrby::IncrBy;
use crate::command::info::Info;
use crate::command::select::Select;
use crate::command::set::Set;
use crate::command::strlen::StrLen;
use crate::command::ttl::Ttl;
use crate::command::Command;
use crate::command::mget::MGet;


use crate::protocol::frame::Frame;
use crate::protocol::ParseError;
use alloc::vec::IntoIter;

#[derive(Debug)]
pub struct Parse {
    pub token: IntoIter<Frame>,
}

impl Parse {
    pub fn new(frame: Frame) -> Result<Parse, ParseError> {
        if let Frame::Array(vec) = frame {
            return Ok(Parse {
                token: vec.into_iter(),
            });
        }
        return Err("invalid ".into());
    }

    pub fn into_command(mut self) -> Result<Command, ParseError> {
        let command_name = self.next()?.into_string()?;

        let cmd = match &command_name.to_lowercase()[..] {
            "ping" => Command::PING,
            "info" => Command::INFO(Info::parse(self)?),
            "get" => Command::GET(Get::parse(self)?),
            "set" => Command::SET(Set::parse(self)?),
            "del" => Command::DEL(Del::parse(self)?),
            "select" => Command::SELECT(Select::parse(self)?),
            "ttl" => Command::TTL(Ttl::parse(self,false)?),
            "pttl" => Command::PTTL(Ttl::parse(self,true)?),
            "exists" => Command::EXISTS(Exists::parse(self)?),
            "incr" =>Command::INCR(IncrBy::parse(self,true)?),
            "incrby" =>Command::INCR(IncrBy::parse(self,false)?),
            "flushdb" => Command::FLUSHDB(Flushdb::parse(self)?),
            "mget" => Command::MGET(MGet::parse(self)?),
            "strlen" => Command::STRLEN(StrLen::parse(self)?),
            _ => Command::UNKNOWN(command_name, self),
        };
        Ok(cmd)
    }

    pub fn next(&mut self) -> Result<Frame, ParseError> {
        self.token.next().ok_or_else(|| ParseError::EOF)
    }

    pub fn remaining(&mut self) -> Result<Vec<Frame>, ParseError> {
        let mut next = self.token.next();
        if next.is_none() {
            return Err(ParseError::EOF);
        }
        let mut vec = vec![];
        while next.is_some() {
            vec.push(next.unwrap());
            next = self.token.next();
        }
        Ok(vec)
    }

    pub fn remaining_into_string_vec(mut self) -> Result<Vec<String>, ParseError> {
        let frames = self.remaining()?;
        let keys = frames
            .into_iter()
            .map(|key| match key.into_string() {
                Ok(s) => Ok(s),
                Err(er) => return Err(er),
            })
            .filter(|s| s.is_ok())
            .map(|f| f.unwrap())
            .collect::<Vec<String>>();
        Ok(keys)
    }

    pub fn fin(&mut self) -> Result<(), ParseError> {
        if self.token.next().is_none() {
            return Ok(());
        }
        Err("expected end of frame,but there is more".into())
    }
}
