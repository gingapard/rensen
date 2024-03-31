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

    let mut des_hosts = Settings::deserialize_yaml("test.yaml")?;

    let mut host_config = &mut des_hosts.hosts[0];
    let mut host = rsync::Rsync::new(&mut host_config);
    let _ = host.full_backup();

    Ok(())
}
