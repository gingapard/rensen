use log::{info as log_info, error};
use std::env;

#[derive(Debug)]
pub enum Info {
    Success(Option<String>),
}

pub fn log_info(info: Info) {
    log_info!("{:?}: {:?}", std::mem::discriminant(&info), info);
}

#[derive(Debug)]
pub enum Error {
    STDLIB(String),
    Undefined(String),
    MisssingFile(String),
    Zip(String),
    Config(String),
    Auth(String),
    Unreachable(String),
    Env(String),
    Backup(String),
    InternalError(String),
}

pub fn log_error(error: Error) {
    error!("{:?}: {:?}", std::mem::discriminant(&error), error);
}

