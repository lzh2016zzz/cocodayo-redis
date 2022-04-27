extern crate fast_log;
extern crate bytes;
extern crate alloc;
extern crate chrono;
#[macro_use]
extern crate lazy_static;


use tokio::net::{TcpListener};
use crate::config::ServerProperties;


mod protocol;
mod server;
mod config;
mod command;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> crate::Result<()> {
    // fast_log::init_log("ledis-server.log", 1000, log::Level::Debug, None, true).unwrap();

    let props = ServerProperties::new();

    let listener = TcpListener::bind(props.get_bind_addr()).await?;

    // log::info!("server listen on {}", props.get_bind_addr());

    server::run(listener,props, tokio::signal::ctrl_c()).await;

    Ok(())
}
