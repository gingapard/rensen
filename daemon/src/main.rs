use rensen_lib::config::*;
use rensen_lib::traits::*;
use rensen_lib::logging::*;

pub mod daemon;
use crate::daemon::*;

use cron::Schedule;
use std::sync::Arc;
use std::path::PathBuf;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Trap>  {
    let global_config_path = PathBuf::from("/etc/rensen/rensen_config.yml");
    let global_config: GlobalConfig = GlobalConfig::deserialize_yaml(&global_config_path)
        .map_err(|err| Trap::FS(format!("Could not deserialize Global Config: {}", err)))?;

    let settings = Settings::deserialize_yaml(&global_config.hosts_path)
        .map_err(|err| Trap::FS(format!("Could not deserialize Settings @ {:?}: {}", global_config.hosts_path, err)))?;

    let mut schedules: Vec<Arc<HostSchedule>> = Vec::new();
    for host in settings.hosts.iter() {
        if let Some(cron_schedule) = &host.config.cron_schedule {
            println!("{:?}", cron_schedule);
            match Schedule::from_str(cron_schedule.as_str()) {
                Ok(schedule) => {
                    println!("sch: {:?}", schedule);
                    
                    let host_schedule = Arc::new(HostSchedule {
                        host: host.clone().into(),
                        schedule,
                    });
                    
                    println!("{:?}", host_schedule);
                    schedules.push(host_schedule);
                },
                Err(err) => {
                    log_trap(&global_config, &Trap::InvalidInput(format!("Invalid Cron Expression for `{}`: {}", host.hostname, err)));
                }
            }
        } else {
            log_trap(&global_config, &Trap::Missing(format!("Missing cron_schedule for `{}`: Defaulting to `0 0 * * *`", &host.hostname)));
            let host_schedule = Arc::new(HostSchedule {
                host: host.clone().into(),
                schedule: Schedule::from_str("0 0 * * *").unwrap(),
            });

            schedules.push(host_schedule);
            continue;
        }
    }

    println!("{:?}", schedules);

    // let backup_scheduler = BackupScheduler::from(Arc::new(global_config), settings, schedules);

    

    println!("Starting scheduler ...");
    //backup_scheduler.run_scheduler().await?;

    Ok(())
}

