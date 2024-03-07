use log::{info, error};
use std::env;

// info
enum Info {
    Success(Option<String>),
}

fn info(info: Info) -> std::io::Result<()> {
    Ok(())
}

// errors
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
    Ok(())
}

