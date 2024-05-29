pub mod record;
pub mod logging;
pub mod compiler;
pub mod utils;
pub use utils::hash_file; pub mod backup; pub mod config; 
pub mod tests;
pub mod traits;
pub mod snapshot;
pub use traits::{Rsync, JsonFile, YamlFile};

use backup::rsync::*;

pub use config::*;
use logging::Trap;

pub use record::Record;
use std::{env, net, io::Result, path::{Path, PathBuf}, error};
// use std::collections::HashMap;
use env_logger;

fn main() -> Result<()> {
    env_logger::init();

    let mut des_hosts = Settings::deserialize_yaml(Path::new("hosts_2.yml"))?;
    


    /*
    let mut entries: HashMap<PathBuf, u64> = HashMap::new();  
    entries.insert("/home/bam/backups/file1".into(), 90);
    entries.insert("/home/bam/backups/file2".into(), 1238947);
    entries.insert("/home/bam/backups/file3".into(), 239847298);
    entries.insert("/home/bam/backups/file4".into(), 2398129837);
    entries.insert("/home/bam/backups/file5".into(), 9812837123);

    let record = Record::new(entries);
    record.serialize_json(Path::new("record.json")).unwrap();
    */

    /*
    let mut config = &mut des_hosts.hosts[0].config;
    let identifier = String::from("192.168.1.97");

    let record = match Record::deserialize_json(&config.destination.join(identifier).join(".records").join("record.json")) {
        Ok(record) => record,
        _ => Record::new()
    };

    let mut host = Sftp::new(&mut config, record, false);
    host.incremental = true;
    host.debug = true;
    let _ = host.backup();
    */

    let path = PathBuf::from("/home/bam/backups/192.168.1.47/.records/2024-05-06-14-49-41Z.json");
    let mut compiler = compiler::Compiler::from(&path).unwrap();

    let dest = PathBuf::from("/home/dto/snapshots");
    let _ = compiler.compile(dest.as_path());

    Ok(())
}
