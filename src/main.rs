pub mod logging;
pub mod utils;
pub mod backup;
pub mod config;

use backup::rsync;

use config::*;
use logging::ErrorType;

use std::{env, net, io::Result, error};
use env_logger;

fn main() -> Result<()> {
    env_logger::init();

    let des_hosts = Settings::deserialize_json("hosts")?;

    let host_config = &des_hosts.hosts[0];
    let host = rsync::Host::new(host_config, None);

    Ok(())
}
