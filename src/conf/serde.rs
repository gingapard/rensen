use serde::{Serialize, Deserialize};
use serde_json;
use std::fs::File;
use std::path::{PathBuf, Path};
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
    pub key_path: Option<PathBuf>,
    pub remote_path: PathBuf,
    pub destination_path: PathBuf,
    pub frequency_hrs: Option<f32>,
}

impl Host {
    pub fn new(
        user: String,
        identifier: HostIdentifier,
        port: u16,
        key_path: PathBuf,
        remote_path: PathBuf,
        destination_path: PathBuf,
        frequency_hrs: f32) -> Self {
        Self {
            user,
            identifier,
            port: Some(port),
            key_path: Some(key_path),
            remote_path,
            destination_path,
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
    
    pub fn verify_syntax_json(file_path: &str) -> std::io::Result<()> {

        Ok(())
    }
}
