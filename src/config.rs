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
pub struct HostConfig {
    pub user: String,
    pub identifier: HostIdentifier, // ip address or domain name
    pub port: Option<u16>, // default: 22
    pub key_path: Option<PathBuf>, // default: "$HOME/.ssh/id_rsa"
    pub remote_path: PathBuf,
    pub dest_path: PathBuf,
    pub frequency_hrs: Option<f32>, // default: 24.0
}

impl HostConfig {
    pub fn new(
        user: String,
        identifier: HostIdentifier,
        port: u16,
        key_path: PathBuf,
        remote_path: PathBuf,
        dest_path: PathBuf,
        frequency_hrs: f32) -> Self {
        Self {
            user,
            identifier,
            port: Some(port),
            key_path: Some(key_path),
            remote_path,
            dest_path,
            frequency_hrs: Some(frequency_hrs),
        }
    }
}

pub struct Settings {
    pub hosts: Vec<HostConfig>,
}

impl Settings {
    pub fn new(hosts: Vec<HostConfig>) -> Self {
        Self {hosts}
    }

    pub fn serialize_json(&self, file_path: &str) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;
        let json_str = serde_json::to_string_pretty(&self.hosts)?;
        write!(file, "{}", json_str)?;
        Ok(())
    }

    pub fn deserialize_json(file_path: &str) -> std::io::Result<Self> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let hosts: Vec<HostConfig> = serde_json::from_str(&contents)?;
        Ok(Self {hosts})
    }
    
    pub fn verify_syntax_json(file_path: &str) -> std::io::Result<()> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(())
    }
}
