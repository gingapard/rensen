use serde::{Serialize, Deserialize};
use serde_json;
use std::fs::File;
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Host {
    hostname: Option<String>,
    user: Option<String>,
    ip: Option<String>,
    key_path: Option<String>,
    remote_path: Option<String>,
    destination_path: Option<String>,
    frequency_hrs: Option<f32>,
}

impl Host {
    pub fn new(
        hostname: String,
        user: String,
        ip: String,
        key_path: String,
        remote_path: String, 
        destination_path: String,
        frequency_hrs: f32) -> Self {
        Self {
            hostname: Some(hostname),
            user: Some(user),
            ip: Some(ip),
            key_path: Some(key_path),
            remote_path: Some(remote_path),
            destination_path: Some(destination_path),
            frequency_hrs: Some(frequency_hrs)
        }
    }
}

pub struct Config {
    config: Vec<Host>,
}

impl Config {
    pub fn new(config: Vec<Host>) -> Self {
        Self {config}
    }

    pub fn serialize_json(&self, file_name: &str) -> std::io::Result<()> {
        let mut file = File::create(file_name)?;
        let serialized_hosts = serde_json::to_string_pretty(&self.config)?;
        write!(file, "{}", serialized_hosts)?;
        Ok(())
    }

    pub fn deserialize_json(file_name: &str) -> std::io::Result<Self> {
        let mut file = File::open(file_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Vec<Host> = serde_json::from_str(&contents)?;
        Ok(Self {config})
    }
}
