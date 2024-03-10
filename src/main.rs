pub mod conf;
pub mod logging;
pub mod utils;
pub mod backup;
use conf::serde;
use std::fs::File;
use std::path::Path;

use std::{env, net, io::Result, error};
use env_logger;

fn main() -> Result<()> {
    env_logger::init();

    let host1 = serde::Host::new(
        "user1".to_string(),
        serde::HostIdentifier::Ip("192.168.1.1".to_string()),
        22,
        "/path/to/key1".to_string(),
        "/remote/path1".to_string(),
        "/destination/path1".to_string(),
        24.0,
    );
    let host2 = serde::Host::new(
        "user2".to_string(),
        serde::HostIdentifier::Hostname("test.com".to_string()),
        22,
        "/path/to/key2".to_string(),
        "/remote/path2".to_string(),
        "/destination/path2".to_string(),
        48.0,
    );

    let config = serde::Config::new(vec![host1, host2]);
    let path = "hosts.conf";
    config.serialize_json(path).unwrap();

    let des_hosts= serde::Config::deserialize_json(path).unwrap();

    let id = &des_hosts.hosts[0].identifier;
    println!("{:?}", id);
    

    Ok(())
}
