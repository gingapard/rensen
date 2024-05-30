use rensen_lib::backup::rsync::Sftp;
use rensen_lib::config::*;
use rensen_lib::traits::*;
use rensen_lib::logging::*;

use chrono::{Local, Timelike};
use cron::Schedule;
use tokio::time::{interval, Duration};
use std::sync::Arc;
use std::thread;
use std::path::{PathBuf, Path};
use std::sync::{Mutex, MutexGuard};

pub struct HostSchedule {
    pub host: Host,
    pub schedule: Schedule,
}

pub struct BackupTask {
    pub global_config: Arc<Mutex<GlobalConfig>>,
    pub host: Host,
}

impl BackupTask {
    fn run_backup_task(&self) -> Result<(), Trap> {

        let global_config = self.global_config.lock().unwrap();
        let settings = Settings::deserialize_yaml(&global_config.backupping_path)
            .map_err(|err| Trap::FS(format!("Could not deserialize Settings @ {:?}: {}", global_config.backupping_path, err)))?;

        Ok(())
    }
}

pub struct RensenDaemon {
    pub global_config: Arc<Mutex<GlobalConfig>>,
    pub settings: Settings,
    pub schedules: Vec<Arc<Mutex<HostSchedule>>>
}

impl RensenDaemon {
    pub fn from(global_config: Arc<Mutex<GlobalConfig>>, settings: Settings, schedules: Vec<Arc<Mutex<HostSchedule>>>) -> Self {
        RensenDaemon { global_config, settings, schedules }
    }

    fn should_run(&self, now: &chrono::DateTime<Local>, host_schedule: &MutexGuard<HostSchedule>) -> bool {
        let current_time = now.with_second(0).unwrap().with_nanosecond(0).unwrap();
        host_schedule.schedule.upcoming(Local).take(1).any(|time| time == current_time)
    }

    /// Check every 60 seconds if it is time to backup.
    pub fn run_scheduler(&self) -> Result<(), Trap>  {
        let mut interval = interval(Duration::from_secs(60));

        loop {

            interval.tick();
            let now = Local::now();

            for host_schedule in self.schedules.iter() {
                let host_schedule = host_schedule.lock().unwrap();
                if self.should_run(&now, &host_schedule) {

                    let _ = thread::spawn(|| {
                        let backup_task = BackupTask { global_config: self.global_config, host: host_schedule.host };
                        if let Err(err) = backup_task.run_backup_task() {
                            log_trap(&self.global_config.lock().unwrap(), &err);
                        }
                    });
                }
            }
        }
    }
}
