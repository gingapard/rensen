use serde::{Serialize, Deserialize};
use serde_json;
use std::fs::File;
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Config {
    hostname: String,
    user: String,
    ip: String,
    key_path: String,
    remote_path: String,
    destination_path: String,
    frequency_hrs: Option<usize>,
}

impl Config {
    pub fn new(
        hostname: String,
        user: String,
        ip: String,
        key_path: String,
        remote_path: String, 
        destination_path: String,
        frequency_hrs: usize) -> Self {
        Self {
            hostname,
            user,
            ip,
            key_path,
            remote_path,
            destination_path,
            frequency_hrs: Some(frequency_hrs)
        }
    }
}

pub struct RemoteHost {
    remote_hosts: Vec<Config>,
}

impl RemoteHost {
    pub fn new(remote_hosts: Vec<Config>) -> Self {
        Self { remote_hosts }
    }

    pub fn serialize_json(&self, file_name: &str) -> std::io::Result<()> {
        let mut file = File::create(file_name)?;
        let serialized_hosts = serde_json::to_string_pretty(&self.remote_hosts)?;
        write!(file, "{}", serialized_hosts)?;
        Ok(())
    }

    pub fn deserialize_json(file_name: &str) -> std::io::Result<Self> {
        let mut file = File::open(file_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let remote_hosts: Vec<Config> = serde_json::from_str(&contents)?;
        Ok(Self { remote_hosts })
    }
}
