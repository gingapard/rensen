use serde::{Serialize, Deserialize};
use serde_json;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum HostIdentifier {
    Ip(String),
    Hostname(String),
}

#[derive(Serialize, Deserialize)]
pub struct Host {
    pub user: String,
    pub identifier: HostIdentifier,
    pub port: Option<u16>,
    pub key_path: Option<String>,
    pub remote_path: String,
    // pub exclude: Option<Vec<String>>,
    pub destination_path: Option<String>,
    pub frequency_hrs: Option<f32>,
    // pub incremental: Option<bool>,
    // pub compression_algorithm: Option<CompressionAlgorithm>,
    // pub encryption_algorithm: Option<EncryptionAlgorithm>,
    // pub retention_policy: Option<RetentionPolicy>,
    // pub connection_timeout_secs: Option<u64>,

}

impl Host {
    pub fn new(
        user: String,
        identifier: HostIdentifier,
        port: u16,
        key_path: String,
        remote_path: String, 
        destination_path: String,
        frequency_hrs: f32) -> Self {
        Self {
            user,
            identifier,
            port: Some(port),
            key_path: Some(key_path),
            remote_path,
            destination_path: Some(destination_path),
            frequency_hrs: Some(frequency_hrs),
        }
    }
}

pub struct Config {
    pub hosts: Vec<Host>,
}

impl Config {
    pub fn new(hosts: Vec<Host>) -> Self {
        Self {hosts}
    }

    // json
    pub fn serialize_json(&self, file_name: &str) -> std::io::Result<()> {
        let mut file = File::create(file_name)?;
        let json_str = serde_json::to_string_pretty(&self.hosts)?;
        write!(file, "{}", json_str)?;
        Ok(())
    }

    pub fn deserialize_json(file_name: &str) -> std::io::Result<Self> {
        let mut file = File::open(file_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let hosts: Vec<Host> = serde_json::from_str(&contents)?;
        Ok(Self {hosts})
    }
}
