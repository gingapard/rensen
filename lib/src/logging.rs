use std::fs::{OpenOptions, File};
use std::path::Path;
use std::io::prelude::*;

use log::error;
use crate::utils::get_datetime;
use crate::config::GlobalConfig;

#[derive(Debug)]
pub enum Trap {

    // rensen lib
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

    // ctl
    InvalidInput(String),
    ReadInput(String)
}

pub enum LogStatus {
    Success,
    Error,
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
    };
    
    // Opening log file
    if !Path::new(&global_config.log_path).exists() {
        let _ = File::create(&global_config.log_path);
    }

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&global_config.log_path)
        .unwrap();

    let current_time = get_datetime();

    if let Err(err) = writeln!(file, "[{}] {}", current_time, trap_msg) {
        eprintln!("Problems writing to log file `{:?}`. Please check permissions: {}", &global_config.log_path, err);
    }

    error!("{}", trap_msg);
}

