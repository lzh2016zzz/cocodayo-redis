use tokio::io::{BufWriter, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use bytes::{BytesMut, Buf};
use crate::protocol::frame::*;
use std::io::Cursor;

use std::net::SocketAddr;
use crate::protocol::FrameError;

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    addr: SocketAddr,
}

impl Drop for Connection {
    fn drop(&mut self) {
        log::debug!("connection closed : {}",self.addr)
    }
}

impl Connection {
    pub fn new(tcp_stream: TcpStream) -> Connection {
        let addr = tcp_stream.peer_addr().unwrap();
        Connection {
            addr,
            stream: BufWriter::new(tcp_stream),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn write_and_flush_frame(&mut self, frame: Frame) -> crate::Result<()> {
        log::debug!("write frame : {:?}",&frame);
        self.write_frame(frame).await?;
        self.flush().await
    }

    pub async fn flush(&mut self) -> crate::Result<()> {
        Ok(self.stream.flush().await?)
    }

    pub async fn write_frame(&mut self, frame: Frame) -> crate::Result<()> {
        let b = frame.into_bytes()?;
        let _ = self.stream.write_all(&b[..]).await?;
        Ok(())
    }

    pub async fn read_frame(&mut self) -> crate::Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err("connection reset by peer".into())
                };
            }
        }
    }

    fn parse_frame(&mut self) -> crate::Result<Option<Frame>> {
        let mut buffer = Cursor::new(&self.buffer[..]);

        return match check(&mut buffer) {
            Ok(_) => {
                let len = buffer.position() as usize;

                buffer.set_position(0);

                let frame = parse_frame(&mut buffer)?;

                self.skip_buf(len)?;

                Ok(Some(frame))
            }
            Err(FrameError::Incomplete) => Ok(None),
            Err(e) => Err(e.into())
        };
    }

    fn skip_buf(&mut self, len: usize) -> crate::Result<()> {
        if self.buffer.remaining() < len {
            return Err("skip_buf > remaining".into());
        }
        self.buffer.advance(len);
        Ok(())
    }
}