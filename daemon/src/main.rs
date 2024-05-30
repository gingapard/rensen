use rensen_lib::backup::rsync::Sftp;
use rensen_lib::config::*;
use rensen_lib::traits::*;
use rensen_lib::logging::*;

pub mod daemon;
use crate::daemon::*;

use std::thread;
use cron::Schedule;
use std::sync::Arc;
use std::path::{PathBuf, Path};
use std::str::FromStr;
use tokio::sync::{Mutex, MutexGuard};

#[tokio::main]
async fn main() -> Result<(), Trap>  {

    let global_config_path = PathBuf::from("/etc/rensen/rensen_config.yml");
    let global_config: GlobalConfig = GlobalConfig::deserialize_yaml(&global_config_path)
        .map_err(|err| Trap::FS(format!("Could not deserialize Global Config: {}", err)))?;

    let settings = Settings::deserialize_yaml(&global_config.backupping_path)
        .map_err(|err| Trap::FS(format!("Could not deserialize Settings @ {:?}: {}", global_config.backupping_path, err)))?;

    let mut schedules: Vec<Arc<Mutex<HostSchedule>>> = Vec::new();
    for host in settings.hosts.iter() {
        if let Some(cron_schedule) = host.config.cron_schedule {
            match Schedule::from_str(cron_schedule.as_str()) {
                Ok(schedule) => {
                   schedules.push(Arc::new(Mutex::new(HostSchedule {
                       host: host.clone(),
                       schedule,
                   })));
                },
                Err(err) => {
                    log_trap(&global_config, &Trap::InvalidInput(format!("Invalid Cron Expression for `{}`: {}", host.hostname, err)));
                }
            }
        }
        else {
            log_trap(&global_config, &Trap::Missing(format!("Missing cron_schedule for `{}`: Defaulting to `0 0 * * *`", &host.hostname)));
            continue;
        }
    }

    let rensen_daemon = RensenDaemon::from(global_config, settings, schedules);
    rensen_daemon.run_scheduler().await;

    Ok(())
}


