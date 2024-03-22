pub mod logging;
pub mod utils;
pub use utils::hash_file;
pub mod backup;
pub mod config;
pub mod tests;

use backup::{rsync, rsync::BackupMethod};

use config::*;
use logging::ErrorType;

use std::{env, net, io::Result, error};
use env_logger;

fn main() -> Result<()> {
    env_logger::init();

    let des_hosts = Settings::deserialize_yaml("test.yaml")?;

    let host_config = &des_hosts.hosts[0];
    let mut host = rsync::Rsync::new(host_config, None);
    host.full_backup();

    Ok(())
}
