use rensen_lib::backup::rsync::Sftp;
use rensen_lib::config::*;
use rensen_lib::traits::*;
use rensen_lib::logging::*;

pub mod daemon;
use crate::daemon::RensenDaemon;

use std::thread;
use std::sync::Arc;
use std::path::{PathBuf, Path};

#[derive(Debug)]
struct RensenSchedule {
    pub schedules: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Trap>  {

    let global_config_path = PathBuf::from("/etc/rensen/rensen_config.yml");
    let global_config: GlobalConfig = GlobalConfig::deserialize_yaml(&global_config_path)
        .map_err(|err| Trap::FS(format!("Could not deserialize Global Config: {}", err)))?;

    let settings = Settings::deserialize_yaml(&global_config.backupping_path)
        .map_err(|err| Trap::FS(format!("Could not deserialize Settings @ {:?}: {}", global_config.backupping_path, err)))?;

    for host in settings.hosts.iter() {

    }

    /*
    let host = &settings.hosts[1];

    let schedule = Schedule::from_str(&host.config.cron_schedule.clone().unwrap_or(String::from("0 0 * * *")))
        .map_err(|err| Trap::InvalidInput(format!("Failed to init scheduler: {}", err)))?;

    let schedule = Arc::new(Mutex::new(schedule));
    let rensen_daemon = RensenDaemon::from(global_config, host.clone(), schedule);

    let _ = rensen_daemon.run_scheduler().await;
    */

    Ok(())
}


