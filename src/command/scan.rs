use crate::{
    protocol::{
        frame::{Frame},
        parse::Parse,
        ParseError,
    },
    server::{shared::Shared},
};

use super::Execable;

#[derive(Debug)]
pub struct Scan {
    cursor: usize,
    pattern: Option<String>,
    cnt: Option<usize>,
}

impl Scan {
    pub fn parse(mut parse: Parse) -> Result<Scan, ParseError> {
        let cursor = parse.next()?.into_decimal()?;
        if cursor < 0 {
            return Err("ERR invalid cursor".into());
        }
        let cursor = cursor as usize;

        let mut pattern = None;

        let mut cnt = None;

        loop {
            match parse.next() {
                Ok(frame) => {
                    let s = frame.into_string()?.to_uppercase();
                    match s.as_str() {
                        "MATCH" => {
                            let patterns = parse.next()?.into_string()?;
                            pattern = Some(patterns);
                        }
                        "COUNT" => {
                            let count = parse.next()?.into_decimal()?;
                            if count < 0 {
                                return Err("ERR invalid count".into());
                            }
                            cnt = Some(count as usize);
                        }
                        _ => return Err("ERR syntax error".into()),
                    }
                }
                Err(ParseError::EOF) => break,
                Err(err) => return Err(err.into()),
            }
        }

        Ok(Scan {
            cursor: cursor,
            pattern: pattern,
            cnt: cnt,
        })
    }
}

impl Execable for Scan {
    fn apply(self, shared: &mut Shared) -> crate::Result<Option<Frame>> {
        let pattern = self.pattern.as_deref();
        let cnt = self.cnt.unwrap_or(0);

        let (scan_result, cursor_result) = shared.scan_for(pattern, self.cursor as i32, cnt);
        let mut result: Vec<Frame> = Vec::with_capacity(2);

        let cursor: Frame = Frame::Str(cursor_result.to_string().into_bytes());
        result.push(cursor);

        let mut scan_frames = Vec::with_capacity(scan_result.len());
        for value_ref in scan_result.into_iter() {
            scan_frames.push(value_ref.frame()?)
        }
        result.push(Frame::Array(scan_frames));

        Ok(Some(Frame::Array(result)))
    }
}
