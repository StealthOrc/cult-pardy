use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use serde::{Deserialize, Serialize};
use wasm_lib::DiscordUser;
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::string::ToString;


pub mod dto;
pub mod backend;
pub mod wasm_lib;



pub fn parse_addr_str(domain: &str, port: usize) -> SocketAddr {
    let addr = format!("{}:{}", domain, port);
    println!("Parsed address: {}", addr);
    let addr = addr.parse::<SocketAddr>().expect("Failed to parse address");
    addr
}

pub fn avatar_image_url2(discord_user: &DiscordUser) -> String {
    format!(
        "https://cdn.discordapp.com/avatars/{}/{}.jpg",
        discord_user.discord_id.id(), discord_user.avatar_id
    )
}






#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct JsonPrinter {
    pub results: HashMap<String, bool>,
}

impl JsonPrinter {
    pub fn new() -> Self {
        JsonPrinter {
            results: HashMap::new(),
        }
    }

    pub fn add_string(&mut self, text: String, result: bool) {
        self.results.insert(text, result);
    }

    pub fn add(&mut self, text: &str, result: bool) {
        self.results.insert(text.to_string(), result);
    }
}

pub fn compress(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(data)?;
    encoder.finish().map_err(|e| e.into())
}

pub fn decompress(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(data);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}

pub fn get_false() -> bool {
    false
}

pub fn get_true() -> bool {
    true
}