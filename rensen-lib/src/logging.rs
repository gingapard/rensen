use log::error;

#[derive(Debug)]
pub enum Trap {

    // rensen lib
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
    Missing,

    // ctl
    ReadInput
}

pub fn log_trap(trap: Trap, msg: &str) {
    error!("{:?}: {}", std::mem::discriminant(&trap), msg);
}

