use rensen_lib::logging::Trap;
use rensen_lib::config::*;
use rensen_lib::traits::YamlFile;

use crate::utils::*;

use std::path::PathBuf;

#[derive(PartialEq)]
pub enum ActionType {
    AddHost,    // (1 arg)
    RemoveHost, // (1 arg)
    ModifyHost, // (1 arg)
    RunBackup,  // (2 arg)
    Compile,    // (2 arg)
    List,       // (2 arg)
    Help,       // (0 arg)
    Exit,       // (0 arg)
}

pub struct Action {
    pub action_type: ActionType,
    pub operands: Vec<String>,

    pub global_config: GlobalConfig,
}

impl Action {
    pub fn execute(&self) -> Result<(), Trap> {

        match self.action_type {
            ActionType::AddHost => {
                self.add_host()?;
            },
            // TODO: add rest of actions.

            _ => (),
        }

        Ok(())
    }

    fn add_host(&self) -> Result<(), Trap> {

        let host_config = self.global_config.hostconfig;
        
        // Global host-settings for rensen
        let mut settings: Settings = Settings::deserialize_yaml(&host_config)
            .map_err(|_| Trap::ReadInput)?;

        // Read addr
        let identifier    = get_input("identifier (addr): ")
            .map_err(|_| Trap::ReadInput)?.trim().to_string();
        
        // Read Username 
        let user =          get_input("user: ")
            .map_err(|_| Trap::ReadInput)?.trim().to_string();

        // Read port
        let port = match get_input("port (leave empty for 22): ") {
            Ok(input) => {
                if input.trim().is_empty() {
                    22
                }
                else {
                    match input.trim().parse::<u16>() {
                        Ok(port) => port,
                        Err(_) => {
                            return Err(Trap::ReadInput);
                        }
                    }
                }
            }
            Err(_) => {
                println!("Failed to read input");
                return Err(Trap::ReadInput);
            }
        };

        // Read key-path
        let key_path      = get_input("ssh-key: ")
            .map_err(|_| Trap::ReadInput)?.trim().to_string();

        // Read source directory
        let source        = get_input("source: ")
            .map_err(|_| Trap::ReadInput)?.trim().to_string();

        // Read destination/backup directory
        let destination   = get_input("destination: ")
            .map_err(|_| Trap::ReadInput)?.trim().to_string();

        // Read backup frequency
        let frequency_hrs = match get_input("backup frquency (hrs): ") {
            Ok(input) => {
                if input.trim().is_empty() {
                    24.0
                }
                else {
                    match input.trim().parse::<f32>() {
                        Ok(port) => port,
                        Err(_) => {
                            return Err(Trap::ReadInput);
                        }
                    }
                }
            }
            Err(_) => {
                println!("Failed to read input");
                return Err(Trap::ReadInput);
            }

        };

        // Add Config to settings and serialize
        let hostconfig = HostConfig::from(user.to_string(), identifier.to_string(), port, PathBuf::from(key_path), PathBuf::from(source), PathBuf::from(destination), frequency_hrs);
        settings.hosts.push(Host { hostname: self.operands[0].clone(), config: hostconfig  });
        
        let _ = settings.serialize_yaml(&host_config)
            .map_err(|_| Trap::FS)?;

        Ok(())
    }
}




