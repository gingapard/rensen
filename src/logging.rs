use log::{info as log_info, error};
use std::env;

#[derive(Debug)]
enum Info {
    Success(Option<String>),
}

fn info(info: Info) -> std::io::Result<()> {
    log_info!("{:?}: {:?}", std::mem::discriminant(&info), info);
    Ok(())
}

#[derive(Debug)]
enum Error {
    Undefined(String),
    Files(String),
    Config(String),
    Auth(String),
    Unreachable(String),
    Env(String),
    Backup(String),
    InternalError(String),
}

fn error(error: Error) -> std::io::Result<()> {
    error!("{:?}: {:?}", std::mem::discriminant(&error), error);
    Ok(())
}

