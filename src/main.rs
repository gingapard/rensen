pub mod record;
pub mod logging;
pub mod compiler;
pub mod utils;
pub use utils::hash_file;
pub mod backup; 
pub mod config; 
pub mod tests;
pub mod traits;
pub mod snapshot;
pub use traits::{Rsync, FileSerializable};

use backup::rsync::*;

pub use config::*;
use logging::Trap;

pub use record::Record;
use std::{env, net, io::Result, path::{Path, PathBuf}, error};
// use std::collections::HashMap;
use env_logger;

fn main() -> Result<()> {
    env_logger::init();

    // let mut des_hosts = Settings::deserialize_yaml(Path::new("hosts_2.yml"))?;
    


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
    let mut host_config = &mut des_hosts.hosts[0];
    let identifier = match &host_config.identifier {
        HostIdentifier::Ip(ip) => ip,
        HostIdentifier::Hostname(hostname) => hostname,
    };

    let record = Record::deserialize_json(&host_config.destination.join(identifier).join(".records").join("record.json"));
    let mut host = Sftp::new(&mut host_config, record.unwrap(), false);
    host.incremental = true;
    host.debug = true;
    let _ = host.backup();
    */

    let mut compiler = compiler::Compiler::from("/home/bam/backups/192.168.1.97/.records/2024-05-08-08-35-00Z.json").unwrap();
    println!("{}", compiler.snapshot);

    let dest = PathBuf::from("/home/bam/snapshots/");
    let _ = compiler.compile(dest.as_path());


    Ok(())
}
