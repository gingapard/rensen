use log::{info as log_info, error};
use std::env;

#[derive(Debug)]
pub enum InfoType {
    Success(),
}

pub fn log_info(info: InfoType, msg: &str) {
    log_info!("{:?}: {}", std::mem::discriminant(&info), msg);
}

#[derive(Debug)]
pub enum ErrorType {
    Connect,
    Session,
    Handshake,
    KeyLoad,
    Auth,
    Channel,
}

pub fn log_error(error: ErrorType, msg: &str) {
    error!("{:?}: {}", std::mem::discriminant(&error), msg);
}

