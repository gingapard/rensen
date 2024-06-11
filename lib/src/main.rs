pub mod record;
pub mod logging;
pub mod compiler; pub mod utils;
pub use utils::hash_file; 
pub mod backup;
use crate::backup::rsync::Sftp;
pub mod config; 
pub mod tests;
pub mod traits;
pub mod snapshot;
pub use traits::{Rsync, JsonFile, YamlFile};


pub use config::*;

pub use record::Record; use std::{env, net, io::Result, path::{Path, PathBuf}, error};
// use std::collections::HashMap;

fn main() -> Result<()> {

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

    let global_config_path = Path::new("/etc/rensen/rensen_config.yml");
    let global_config: GlobalConfig = GlobalConfig::deserialize_yaml(global_config_path)?;
    let settings: Settings = Settings::deserialize_yaml(&global_config.hosts)?;
    let host_config = &settings.hosts[1].config;

    let record = match Record::deserialize_json(&global_config.backups.join(&host_config.identifier).join(".records").join("record.json")) {
        Ok(record) => record,
        _ => Record::new()
    };

    let mut host = Sftp::new(&host_config, &global_config, record, false);
    host.incremental = true;
    host.debug = true;
    let _ = host.backup();

    Ok(())
}
