use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;

pub struct ServerProperties {
    pub bind: String,
    pub port: u16,
    pub append_only: bool,
    pub append_filename: String,
    pub max_clients: i32,
}

impl ServerProperties {
    pub(crate) fn new() -> ServerProperties {
        ServerProperties {
            bind: "0.0.0.0".to_string(),
            port: 6379,
            append_only: false,
            append_filename: "./ldb_data".to_string(),
            max_clients: 0,
        }
    }

    pub(crate) fn get_bind_addr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::from_str(self.bind.as_str()).unwrap(), self.port)
    }
}