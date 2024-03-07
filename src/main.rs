mod conf;
mod logging;
use conf::serde;

use std::{env, net, io::Result, error};
use env_logger;

fn main() -> Result<()> {
    env_logger::init();

    let mut config = serde::Config::new("test".to_string(), "test".to_string(), "test".to_string(), "test".to_string(), "test".to_string(), "test".to_string(), 24);
    let mut config2 = serde::Config::new("test".to_string(), "test".to_string(), "test".to_string(), "test".to_string(), "test".to_string(), "test".to_string(), 24);
    let mut config3 = serde::Config::new("test".to_string(), "test".to_string(), "test".to_string(), "test".to_string(), "test".to_string(), "test".to_string(), 24);
    let remote_host = serde::RemoteHost::new(vec![config, config2, config3]);

    remote_host.serialize_json("config.json")?;

    Ok(())
}
