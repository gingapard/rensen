use log::error;

#[derive(Debug)]
pub enum Trap {
    Success,
    STD,
    Connect,
    Session,
    Handshake,
    KeyLoad,
    Auth,
    Channel,
    FS,
    Config,
    Copy,
}

pub fn log_trap(trap: Trap, msg: &str) {
    error!("{:?}: {}", std::mem::discriminant(&trap), msg);
}

