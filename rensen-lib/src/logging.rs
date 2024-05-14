use log::error;

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

pub fn log_trap(trap: Trap) {
    match trap {
        Trap::STD(msg)          => error!("STD: {}", msg),
        Trap::Connect(msg)      => error!("Connect: {}", msg),
        Trap::Session(msg)      => error!("Session: {}", msg),
        Trap::Handshake(msg)    => error!("Handshake: {}", msg),
        Trap::KeyLoad(msg)      => error!("KeyLoad: {}", msg),
        Trap::Auth(msg)         => error!("Auth: {}", msg),
        Trap::Channel(msg)      => error!("Channel: {}", msg),
        Trap::FS(msg)           => error!("FS: {}", msg),
        Trap::Config(msg)       => error!("Config: {}", msg),
        Trap::Copy(msg)         => error!("Copy: {}", msg),
        Trap::Missing(msg)      => error!("Missing: {}", msg),
        Trap::InvalidInput(msg) => error!("Invalid: {}", msg),
        Trap::ReadInput(msg)    => error!("ReadInput: {}", msg),
    }
}

