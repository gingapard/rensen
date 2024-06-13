use rensen_lib::backup::rsync::Sftp;
use rensen_lib::config::*;
use rensen_lib::traits::*;
use rensen_lib::logging::*;
use rensen_lib::record::*;

use std::sync::Arc;

// Struct for running the actual backup task
#[derive(Debug)]
pub struct BackupTask {
    pub global_config: Arc<GlobalConfig>, 
    pub host: Arc<Host>, 
}

impl BackupTask {

    /// Performs backup task using the rensen sftp-backup lib
    pub async fn run(&self) -> Result<(), Trap> {

        let hostname = &self.host.hostname;
        let inc = true;
        let host_config = &self.host.config;

        let record_path = host_config.destination
            .join(&host_config.identifier)
            .join(".records")
            .join("record.json");

        let record = Record::deserialize_json(&record_path)
            .map_err(|err| Trap::FS(format!("Could not read record for host `{}`: {}", hostname, err)))?;

        let mut sftp = Sftp::new(&host_config, &self.global_config, record, inc);

        sftp.incremental = inc;
        sftp.backup()?;

        Ok(())
    }
}
