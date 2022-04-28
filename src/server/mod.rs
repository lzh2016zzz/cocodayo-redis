pub mod connection;
pub mod db;
pub mod handler;
pub mod value_ref;
pub mod shared;

use crate::config::ServerProperties;
use crate::protocol::frame::Frame;
use crate::server::connection::Connection;
use crate::server::db::{Db};
use crate::server::handler::{Handler, ShutdownHook};


use std::future::Future;

use std::sync::{Arc};
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream};

use tokio::sync::mpsc::Receiver;
use tokio::sync::{broadcast, mpsc};
use tokio::time;


pub struct Server {
    tcp_listener: TcpListener,
    db: Arc<Db>,
    pub notify_shutdown: broadcast::Sender<()>,
    pub shutdown_complete_rx: broadcast::Receiver<()>,
}

impl Server {
    pub fn new(
        tcp_listener: TcpListener,
        notify_shutdown: broadcast::Sender<()>,
        shutdown_complete_rx: broadcast::Receiver<()>,
        db: Db,
    ) -> Server {
        Server {
            notify_shutdown,
            shutdown_complete_rx,
            tcp_listener,
            db: Arc::new(db),
        }
    }

    pub async fn run(&mut self) -> crate::Result<()> {
        loop {
            let socket = self.accept().await?;

            log::debug!("accept conn {:?}", socket);

            let connection = Connection::new(socket);


            let db_ptr = Arc::clone(&self.db);

            let mut handler = Handler {
                db: db_ptr,
                connection,
                shutdown: ShutdownHook::new(self.notify_shutdown.subscribe()),
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    log::error!("connection error,message: {}", err);
                }
            });
        }
    }

    async fn accept(&self) -> crate::Result<TcpStream> {
        let mut backoff = 1;
        loop {
            match self.tcp_listener.accept().await {
                Ok((conn, _)) => return Ok(conn),
                Err(e) => {
                    if backoff > 64 {
                        return Err(e.into());
                    }
                }
            }
            time::sleep(Duration::from_secs(backoff)).await;

            backoff *= 2
        }
    }
}

pub async fn run(tcp_listener: TcpListener, props:ServerProperties,shutdown: impl Future) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, _) = broadcast::channel(1);

    let (db, db_worker) = db::Db::new(&props.append_filename,shutdown_complete_tx.subscribe());

    let mut server = Server::new(
        tcp_listener,
        notify_shutdown,
        shutdown_complete_tx.subscribe(),
        db,
    );

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                log::error!("failed to accept connection {}",err);
            }
        }
        worker = db_worker.run() => {
             if let Err(err) = worker {
                 log::error!("db exec error {}",err);
             }
        }
        _ = shutdown => {
            let _ = shutdown_complete_tx.send(());
            // The shutdown signal has been received.
            log::info!("shutting down");
        }
    }


    let Server {
        mut shutdown_complete_rx,
        notify_shutdown,
        ..
    } = server;

    drop(notify_shutdown);


    let _ = shutdown_complete_rx.recv().await;
}
