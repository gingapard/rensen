use log::{info as log_info, error};

#[derive(Debug)]
pub enum InfoType {
    Success,
}

#[derive(Debug)]
pub enum ErrorType {
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

pub fn log_info(info: InfoType, msg: &str) {
    log_info!("{:?}: {}", std::mem::discriminant(&info), msg);
}

pub fn log_error(error: ErrorType, msg: &str) {
    error!("{:?}: {}", std::mem::discriminant(&error), msg);
}

