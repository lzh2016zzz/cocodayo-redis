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
mod logger;
mod banner;
mod utils;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> crate::Result<()> {
    
    logger::setup_logger()?;

    banner::banner_show("");
    
    let props = ServerProperties::new();

    log::info!("{:?}",props);

    let listener = match TcpListener::bind(props.get_bind_addr()).await {
        Ok(value) => value,
        Err(err) => {
            log::error!("{}",err);
            return Ok(());
        },
    };

    log::info!("persistent redis server listen on {}", props.get_bind_addr());

    server::run(listener,props, tokio::signal::ctrl_c()).await;

    Ok(())
}
