pub mod record;
pub mod logging;
pub mod utils;
pub use utils::hash_file;
pub mod backup;
pub mod config;
pub mod tests;
pub mod traits;
pub use traits::{BackupMethod, FileSerializable};

use backup::rsync;

use config::*;
use logging::ErrorType;

use record::Record;
use std::{env, net, io::Result, path::{Path, PathBuf}, error};
use std::collections::HashMap;
use env_logger;

fn main() -> Result<()> {
    env_logger::init();

    let mut des_hosts = Settings::deserialize_yaml(Path::new("test.yaml"))?;
    let mut entries: HashMap<PathBuf, u64> = HashMap::new();  
    entries.insert("/home/bam/backups/file1".into(), 90);
    entries.insert("/home/bam/backups/file2".into(), 1238947);
    entries.insert("/home/bam/backups/file3".into(), 239847298);
    entries.insert("/home/bam/backups/file4".into(), 2398129837);
    entries.insert("/home/bam/backups/file5".into(), 9812837123);

    let record = Record::new(entries);
    record.serialize_json(Path::new("record.json")).unwrap();

    let mut host_config = &mut des_hosts.hosts[0];
    let mut host = rsync::Rsync::new(&mut host_config, record);
    let _ = host.full_backup();

    Ok(())
}
