use std::fs::{OpenOptions, File};
use std::path::Path;
use std::io::prelude::*;

use log::error;
use crate::utils::get_datetime;
use crate::config::GlobalConfig;

#[derive(Debug)]
pub enum Trap {

    STD(String),
    Connect(String),
    Session(String),
    Handshake(String),
    KeyLoad(String),
    Auth(String),
    Channel(String),
    FS(String),
    Config(String),
    Copy(String),
    Missing(String),
    InvalidInput(String),
    ReadInput(String),
    Deserialize(String),
    Serialize(String),
    Metadata(String),
    Scheduler(String),


}

pub fn log_trap(global_config: &GlobalConfig, trap: &Trap) {
    let trap_msg = match trap {
        Trap::STD(msg)          => format!("STD: {}", msg),
        Trap::Connect(msg)      => format!("Connect: {}", msg),
        Trap::Session(msg)      => format!("Session: {}", msg),
        Trap::Handshake(msg)    => format!("Handshake: {}", msg),
        Trap::KeyLoad(msg)      => format!("KeyLoad: {}", msg),
        Trap::Auth(msg)         => format!("Auth: {}", msg),
        Trap::Channel(msg)      => format!("Channel: {}", msg),
        Trap::FS(msg)           => format!("FS: {}", msg),
        Trap::Config(msg)       => format!("Config: {}", msg),
        Trap::Copy(msg)         => format!("Copy: {}", msg),
        Trap::Missing(msg)      => format!("Missing: {}", msg),
        Trap::InvalidInput(msg) => format!("Invalid: {}", msg),
        Trap::ReadInput(msg)    => format!("ReadInput: {}", msg),
        Trap::Serialize(msg)    => format!("Serialize: {}", msg),
        Trap::Deserialize(msg)  => format!("Deserialize: {}", msg),
        Trap::Metadata(msg)     => format!("Metadata: {}", msg),
        Trap::Scheduler(msg)     => format!("Scheduler: {}", msg),
    };
    
    // Opening log file
    if !Path::new(&global_config.log).exists() {
        let _ = File::create(&global_config.log);
    }

    let mut file = match OpenOptions::new()
        .write(true)
        .append(true)
        .open(&global_config.log) {
            Ok(file) => file,
            Err(_) => {
                println!("Hey! you should run this program as sudo if you expect the logging to work.");
                return;
            },
    };

    let current_time = get_datetime();

    if let Err(err) = writeln!(file, "[{}] {}", current_time, trap_msg) {
        eprintln!("Problems writing to log file `{:?}`. Please check permissions: {}", &global_config.log, err);
    }

    error!("{}", trap_msg);
}
