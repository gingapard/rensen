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
use std::fmt;

struct CronSchedule {
    minute: String,
    hour: String,
    day_of_month: String,
    month: String,
    day_of_week: String,
}

impl CronSchedule {
    fn new(
        minute: &str,
        hour: &str,
        day_of_month: &str,
        month: &str,
        day_of_week: &str,
        ) -> Self {
        CronSchedule {
            minute: minute.to_string(),
            hour: hour.to_string(),
            day_of_month: day_of_month.to_string(),
            month: month.to_string(),
            day_of_week: day_of_week.to_string(),
        }
    }

    fn to_cron_string(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.minute, self.hour, self.day_of_month, self.month, self.day_of_week
        )
    }
}

impl fmt::Display for CronSchedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_cron_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Trap>  {

    let cron_schedule = "0 9 * * Mon, Wed *";
    let schedule = Schedule::from_str(cron_schedule)
        .map_err(|err| Trap::InvalidInput(format!("Failed to init scheduler: {}", err)))?;

    let schedule = Arc::new(Mutex::new(schedule));
    run_scheduler(schedule).await;

    Ok(())
}

async fn run_scheduler(schedule: Arc<Mutex<Schedule>>) {
    let mut interval = interval(Duration::from_secs(60));

    loop {

        interval.tick().await;
        let now = Local::now();

        if should_run(&now, &schedule.lock().await) {
            let _ = run_backup_task().await;
        }
    }
}

fn should_run(now: &chrono::DateTime<Local>, schedule: &MutexGuard<Schedule>) -> bool {
    let current_time = now.with_second(0).unwrap().with_nanosecond(0).unwrap();
    schedule.upcoming(Local).take(1).any(|time| time == current_time)
}

async fn run_backup_task() -> Result<(), Trap> {

    let global_config_path = PathBuf::from("/etc/rensen/rensen_config.yml");
    let global_config: GlobalConfig = GlobalConfig::deserialize_yaml(&global_config_path)
        .map_err(|err| Trap::FS(format!("Could not deserialize Global Config: {}", err)))?;

    let settings = Settings::deserialize_yaml(&global_config.backupping_path)
        .map_err(|err| Trap::FS(format!("Could not deserialize Settings @ {:?}: {}", global_config.backupping_path, err)))?;

    Ok(())
}
