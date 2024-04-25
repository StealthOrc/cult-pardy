use std::net::SocketAddr;
use serde::{Deserialize, Serialize};

pub fn parse_addr_str(domain: &str, port: usize) -> SocketAddr {
    let addr = format!("{}:{}", domain, port);
    let addr = addr.parse::<SocketAddr>().expect("Failed to parse address");
    addr
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserSessionRequest{
    pub session_id : usize
}
