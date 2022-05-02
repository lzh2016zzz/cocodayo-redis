use crate::protocol::ParseError;
use crate::protocol::parse::Parse;
use crate::server::db::{CommandResult, Db};
use crate::server::Connection;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::protocol::frame::Frame;

pub struct Handler {
    pub(crate) db: Arc<Db>,
    pub(crate) connection: Connection,
    pub(crate) shutdown: ShutdownHook,
}

impl Handler {
    pub async fn run(&mut self) -> crate::Result<()> {
        while !self.shutdown.is_shutdown() {
            let maybe_frame = tokio::select! {
                res = self.connection.read_frame() => res?,
                _ = self.shutdown.receive() => {
                    // If a shutdown signal is received, return from `run`.
                    // This will result in the task terminating.
                    return Ok(());
                }
            };

            let frame = match maybe_frame {
                Some(frame) => frame,
                None => return Ok(()),
            };

            log::debug!("request frame : {:?}", frame);

            let parse = Parse::new(frame)?;

            match parse.into_command() {
                Ok(cmd) => {
                    if let Some(CommandResult { frame }) = self.db.exec(cmd).await? {
                        self.connection.write_and_flush_frame(frame).await?;
                    } else {
                        self.connection.write_and_flush_frame(Frame::Nil).await?;
                    }
                }
                Err(ParseError::EOF) =>  {
                    self.connection.write_and_flush_frame(Frame::Error("ERR wrong number of arguments for command".to_string())).await?;
                },
                Err(err) => {
                    let err = Frame::Error(format!("{}", err));
                    let _ = self.connection.write_and_flush_frame(err).await?;
                }
            };
        }
        Ok(())
    }
}

pub struct ShutdownHook {
    shutdown: bool,
    notify: broadcast::Receiver<()>,
}

impl ShutdownHook {
    pub fn new(notify: broadcast::Receiver<()>) -> ShutdownHook {
        ShutdownHook {
            shutdown: false,
            notify,
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    pub async fn receive(&mut self) {
        if self.shutdown {
            return;
        }
        let _ = self.notify.recv().await;

        self.shutdown = true;
    }
}
