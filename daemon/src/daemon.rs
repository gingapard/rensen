use rensen_lib::backup::rsync::Sftp;
use rensen_lib::config::*;
use rensen_lib::traits::*;
use rensen_lib::logging::*;

use chrono::{Datelike, Local, Timelike};
use cron::Schedule;
use std::str::FromStr;
use tokio::time::{interval, Duration};
use std::sync::Arc;
use tokio::sync::{MutexGuard, Mutex};
use std::path::{PathBuf, Path};

pub struct BackupTask {
    pub global_config: GlobalConfig,
    pub host: Host,
}

impl BackupTask {
    async fn run_backup_task(&self) -> Result<(), Trap> {
        let settings = Settings::deserialize_yaml(&self.global_config.backupping_path)
            .map_err(|err| Trap::FS(format!("Could not deserialize Settings @ {:?}: {}", self.global_config.backupping_path, err)))?;

        Ok(())
    }
}

pub struct RensenDaemon {
    pub global_config: GlobalConfig,
    pub host: Host,
    pub schedules: Vec<Arc<Mutex<Schedule>>>
}

impl RensenDaemon {
    pub fn from(global_config: GlobalConfig, host: Host, schedules: Vec<Arc<Mutex<Schedule>>>) -> Self {
        RensenDaemon { global_config, host, schedules }
    }

    async fn should_run(&self, now: &chrono::DateTime<Local>) -> bool {
        let current_time = now.with_second(0).unwrap().with_nanosecond(0).unwrap();
        let schedule = self.scheduler.lock().await;

        schedule.upcoming(Local).take(1).any(|time| time == current_time)
    }


    /// Check every 60 seconds if it is time to backup.
    pub async fn run_scheduler(&self) -> Result<(), Trap>  {
        let mut interval = interval(Duration::from_secs(60));

        loop {

            interval.tick().await;
            let now = Local::now();

            if self.should_run(&now).await {
                // let _ = self.run_backup_task().await;
            }
        }
    }
}
