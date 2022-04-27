
use crate::command::Command;
use crate::server::*;

use super::shared::Shared;

pub struct Db {
    cmd_sender: mpsc::Sender<(Command, mpsc::Sender<CommandResult>)>,
}

unsafe impl Send for Db {}

unsafe impl Sync for Db {}

impl Drop for Db {
    fn drop(&mut self) {}
}

impl Db {
    pub fn new(append_file: &str,shutdown_complete_tx: broadcast::Receiver<()>) -> (Db, DbWorker) {
        let hook = ShutdownHook::new(shutdown_complete_tx);
        let (sender, recv) = mpsc::channel::<(Command, mpsc::Sender<CommandResult>)>(1000);
        (Db { cmd_sender: sender }, DbWorker::new(append_file,recv, hook))
    }

    pub(crate) async fn exec(&self, cmd: Command) -> crate::Result<Option<CommandResult>> {
        let (sender, mut rec) = mpsc::channel(1);
        let _ = self.cmd_sender.send((cmd, sender)).await?;
        let recv = rec.recv().await;
        Ok(recv)
    }
}

pub struct DbWorker {
    recv: Receiver<(Command, mpsc::Sender<CommandResult>)>,
    shutdown_hook: ShutdownHook,
    shared: Shared,
}

impl DbWorker {
    pub fn new(
        appendfile :&str,
        recv: Receiver<(Command, mpsc::Sender<CommandResult>)>,
        shutdown_hook: ShutdownHook,
    ) -> DbWorker {
        let shared = Shared::new(appendfile);
        DbWorker {
            recv,
            shutdown_hook,
            shared,
        }
    }

    pub async fn run(mut self) -> crate::Result<()> {
        let mut receiver = self.recv;

            while !self.shutdown_hook.is_shutdown() {
                let maybe_cmd = tokio::select! {
                    res = receiver.recv() => res,
                    _ = self.shutdown_hook.receive() => {
                        return Ok(());
                    }
                };

                if let Some((cmd, callback)) = maybe_cmd {
                    let result = match cmd.apply(&mut self.shared).await {
                        Ok(res) => res,
                        Err(err) => Frame::Error(format!("{}", err)),
                    };

                    let _ = callback.send(CommandResult { frame: result }).await;
                }
            }
            Ok(())
    }
}

pub struct CommandResult {
    pub frame: Frame,
}
