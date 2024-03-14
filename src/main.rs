pub mod conf;
pub mod logging;
pub mod utils;
pub mod backup;
use conf::serde;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use std::{env, net, io::Result, error};
use env_logger;

fn main() -> Result<()> {
    env_logger::init();

    let des_hosts= serde::Config::deserialize_json("hosts.conf").unwrap();

    let host = &des_hosts.hosts[0];
    println!("deserialized");
    backup::backup_rsync(&host).unwrap();

    Ok(())
}
