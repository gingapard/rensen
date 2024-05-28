use serde::{Serialize, Deserialize};
use serde_json;
use serde_yaml;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{self, Write, Read};
use std::fmt;

use crate::traits;
use traits::YamlFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub hosts_path: PathBuf,
    pub backupping_path: PathBuf,
    pub snapshots_path: PathBuf,
    pub log_path: PathBuf,
}

#[test]
fn test_global_config_serialize() {
    let gc = GlobalConfig {
        hosts_path: PathBuf::from("/etc/rensen/hosts.yml"),
        backupping_path: PathBuf::from("/home/dto/bakcups/"),
        snapshots_path: PathBuf::from("/etc/rensen/hosts.yml"),
        log_path: PathBuf::from("/etc/rensen/log"),
    };

    let path = PathBuf::from("gc.yml");
    let _ = gc.serialize_yaml(&path);
}

impl YamlFile for GlobalConfig {
    fn serialize_yaml(&self, file_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;
        let yaml_str = serde_yaml::to_string(&self)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        let formatted_yaml = serde_yaml::to_string(&serde_yaml::from_str::<serde_yaml::Value>(&yaml_str)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))? 
        ).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        file.write_all(formatted_yaml.as_bytes())?;
        Ok(())
    }

    fn deserialize_yaml(file_path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let conf: GlobalConfig = serde_yaml::from_str(&contents)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        Ok(conf)
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub user: String,
    pub identifier: String,        // machine addr
    pub port: Option<u16>,         // default: 22
    pub key_path: Option<PathBuf>, // default: "$HOME/.ssh/ed25516"
    pub source: PathBuf,
    pub destination: PathBuf,
    pub cron_schedule: Option<String>, // defualt `0 0 * * *`
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Host {
    pub hostname: String,
    pub config: HostConfig
}

pub struct Settings {
    pub hosts: Vec<Host>,
}

impl HostConfig {

    pub fn from(
        user: String,
        identifier: String,
        port: u16,
        key_path: PathBuf,
        source: PathBuf,
        destination: PathBuf,
        cron_schedule: String
        ) -> Self {
        Self {
            user,
            identifier,
            port: Some(port),
            key_path: Some(key_path),
            source,
            destination,
            cron_schedule: Some(cron_schedule),
        }
    }
}

impl fmt::Display for HostConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "addr: {}\nuser: {}\nport: {}\nkey Path: {}\nsource: {}\ndestination: {}\nfrequency (hrs): {}",
            self.identifier,
            self.user,
            self.port.unwrap_or(22),
            self.key_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "$HOME/.ssh/ed25516".to_string()),
            self.source.display(),
            self.destination.display(),
            self.cron_schedule.as_ref().unwrap(),
        )
    }
}

impl Settings {
    pub fn new(hosts: Vec<Host>) -> Self {
        Self { hosts }  
    }

    pub fn verify_syntax_json(file_path: &Path) -> std::io::Result<()> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let _: Vec<HostConfig> = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(err) => {
                return Err(err.into());
            }
        };

        Ok(())
    }

    pub fn associated_config(&self, hostname: &String) -> Option<HostConfig> {
        let mut host_config: Option<HostConfig> = None;
        for host in &self.hosts {
            if hostname.to_owned() == host.hostname.to_owned() {
                host_config = Some(host.config.clone());
                break;
            }
        }

        host_config
    }
}

impl YamlFile for Settings {
    fn serialize_yaml(&self, file_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;
        let yaml_str = serde_yaml::to_string(&self.hosts)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        let formatted_yaml = serde_yaml::to_string(&serde_yaml::from_str::<serde_yaml::Value>(&yaml_str)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))? 
        ).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        file.write_all(formatted_yaml.as_bytes())?;
        Ok(())
    }

    fn deserialize_yaml(file_path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let hosts: Vec<Host> = serde_yaml::from_str(&contents)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        Ok(Self { hosts })
    }
}
