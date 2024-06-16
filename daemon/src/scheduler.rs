use rensen_lib::config::*;
use rensen_lib::logging::*;

use chrono::{Local, Timelike, SecondsFormat};
use cron::Schedule;
use tokio::time::{interval, Duration};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::utils::*;
use crate::tasks::*;

// Struct for holding the host data with it's associate schedul
// Wrapper for cron::Schedule
#[derive(Debug)]
pub struct WSchedule {
    pub host: Arc<Host>, 
    pub schedule: Schedule,
}

pub struct Scheduler {
    pub global_config: Arc<GlobalConfig>, 
    pub settings: Settings,
    pub schedules: Vec<Arc<WSchedule>>,
    queue: Arc<Mutex<TaskQueue<BackupTask>>>
}

impl Scheduler {
    pub fn from(global_config: Arc<GlobalConfig>, settings: Settings, schedules: Vec<Arc<WSchedule>>) -> Self {
        Scheduler { global_config, settings, schedules, queue: Arc::new(Mutex::new(TaskQueue::new())) }
    }

    pub async fn run_executor(&mut self) -> Result<(), Trap> {
        let mut interval = interval(Duration::from_secs(60));

        // Execute while there are tasks in queue
        loop {
            interval.tick().await;

            if let Some(task) = self.queue.lock().unwrap().peek() {
                let _ = task.run();

                self.queue.lock().unwrap().popf();
            }
        }

        Ok(())
    }

    // Locks mutex and returns first available task if Some()
    fn get_next_task(&self) -> Option<BackupTask> {
        let mut queue = self.queue.lock().unwrap();
        queue.popf()
    }

    /// Checking according to the hosts's schedule if it is time to backup at this moment.
    fn should_run(&self, now: &chrono::DateTime<Local>, host_schedule: &WSchedule) -> bool {
        let current_time = now
        .with_second(0).unwrap()
        .with_nanosecond(0).unwrap();

        let mut upcoming_times = host_schedule.schedule.upcoming(Local).take(1);

        if let Some(scheduled_time) = upcoming_times.next() {
            println!(
                "Current time: {} (h: {}, m: {}, s: {}), Scheduled time: {} (h: {}, m: {}, s: {})",
                current_time.to_rfc3339_opts(SecondsFormat::Secs, true),
                current_time.hour(), current_time.minute(), current_time.second(),
                scheduled_time.to_rfc3339_opts(SecondsFormat::Secs, true),
                scheduled_time.hour(), scheduled_time.minute(), scheduled_time.second()
            );

            // Compare up to minutes precision
            return current_time == scheduled_time.with_second(0).unwrap().with_nanosecond(0).unwrap();
        }
        false
    }

    /// Looping through the schedules and running eventual backup tasks
    /// when self.should_run() == true
    /// Will wait 60 seconds between each check
    pub async fn run_scheduler(&mut self) -> Result<(), Trap> {
        let mut interval = interval(Duration::from_secs(60));

        
        loop {

            // Checking every interval if it's time
            interval.tick().await;
            let now = Local::now();

            for schedule in self.schedules.iter() {
                if self.should_run(&now, &schedule) {

                    let global_config_clone = Arc::clone(&self.global_config);
                    let host = Arc::clone(&schedule.host); 
                    let backup_task = BackupTask { global_config: global_config_clone, host };

                    // self.queue.lock().unwrap().pushb(backup_task);

                    // Spawning new thread as it's time for backupping
                    tokio::spawn(async move {
                        if let Err(err) = backup_task.run().await {
                            log_trap(&backup_task.global_config, &err); 
                        }
                    });
                }
            }
        }
    }
}

