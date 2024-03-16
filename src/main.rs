pub mod logging;
pub mod utils;
pub mod backup;
pub mod config;

use config::*;
use logging::ErrorType;

use std::{env, net, io::Result, error};
use env_logger;

fn main() -> Result<()> {
    env_logger::init();

    let des_hosts = Settings::deserialize_json("hosts")?;

    let host = &des_hosts.hosts[0];
    backup::rsync::backup(&host).unwrap();

    Ok(())
}
