use rensen_lib::logging::Trap; 
use rensen_lib::config::*;
use rensen_lib::traits::{YamlFile, JsonFile, Rsync};
use rensen_lib::backup::rsync::Sftp;
use rensen_lib::record::Record;
use rensen_lib::compiler::Compiler;

use console::Style;

use std::time::SystemTime;
use crate::utils::*;
use std::path::PathBuf; use std::fs;

#[derive(PartialEq)]
pub enum BackupMethod {
    Full,
    Incremental,
}

#[derive(PartialEq)]
pub enum ListMethod {
    Snapshots,
    Config,
}

#[derive(PartialEq)]
pub enum ActionType {
    AddHost,    // 1 arg
    DeleteHost, // 1 arg
    ModifyHost, // 1 arg
    RunBackup,  // 2 arg
    Compile,    // 1 arg
    List,       // 2 arg

    Clear,      // 0 arg
    Help,       // 0 arg
    Exit,       // 0 arg
}

pub struct Action {
    pub action_type: ActionType,
    pub operands: Vec<String>,
    pub global_config: GlobalConfig,
}

impl Action {
    pub fn execute(&self) -> Result<(), Trap> {

        match self.action_type {
            ActionType::AddHost    => {
                self.add_host()?;
            },
            ActionType::DeleteHost => {
                self.del_host()?;
            },
            ActionType::ModifyHost => {
                self.mod_host()?;
            },
            ActionType::RunBackup  => {
                self.run_backup()?;
            },
            ActionType::Compile    => {
                self.compile_snapshot()?;
            },
            ActionType::List       => {
                self.list()?;
            },
            ActionType::Help       => {
                self.print_help();
            }

            _ => (),
        }

        Ok(())
    }

    /* add action */

    fn add_host(&self) -> Result<(), Trap> {
        if self.operands.len() != 1 {
            return Err(
                Trap::InvalidInput(
                    String::from("Invalid arguments for action. Use `help` for more details")
                )
            );
        }

        let hosts = &self.global_config.hosts;
        let hostname = &self.operands[0];

        let mut settings: Settings = Settings::deserialize_yaml(&hosts)
            .map_err(|err| Trap::Deserialize(format!("Could not deserialize settings: {}", err)))?;

        // checking if the hostname is taken
        for host in settings.hosts.iter() {
            if hostname.to_owned() == host.hostname {
                return Err(Trap::InvalidInput(format!("Hostname `{}` already in use!", hostname)));
            }
        }

        // Read addr
        let identifier = get_input("addr: ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?.trim().to_string();
        
        // Read Username 
        let user = get_input("user: ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?.trim().to_string();

        // Read port
        let port = match get_input("port (press enter for 22): ") {
            Ok(input) => {
                if input.trim().is_empty() {
                    22
                }
                else {
                    match input.trim().parse::<u16>() {
                        Ok(port) => port,
                        Err(err) => {
                            return Err(
                                Trap::ReadInput(format!("Could not read input: {}", err))
                            );
                        }
                    }
                }
            }
            Err(err) => {
                println!("Failed to read input");
                return Err(Trap::ReadInput(format!("Could not read input: {}", err)));
            }
        };

        // Read key-path
        let key= get_input("ssh-key path: ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?
            .trim().to_string();

        // Read source directory
        let source = get_input("source: ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?
            .trim().to_string();

        // Read destination/backup directory
        let destination = get_input("destination: ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?
            .trim().to_string();

        // Read backup schedule
        let cron_schedule = get_input("backupping schedule (Cron expression): ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?
            .trim().to_string();
        // Add Config to settings and serialize
        let host_config = HostConfig::from(user.to_string(), identifier.to_string(), port, PathBuf::from(key), PathBuf::from(source), PathBuf::from(destination), cron_schedule.to_string());
        println!("{}", &host_config);

        settings.hosts.push(Host { hostname: hostname.clone(), config: host_config  });

        let _ = settings.serialize_yaml(hosts)
            .map_err(|err| Trap::Serialize(format!("Could not serialize yaml: {}", err)))?;

        Ok(())
    }

    /* del action */

    fn del_host(&self) -> Result<(), Trap> {
        if self.operands.len() != 1 {
            return Err(
                Trap::InvalidInput(
                    String::from("Invalid arguments for action. Use `help` for more details")
                )
            );
        }
        
        let hostname = &self.operands[0];
        let hosts = &self.global_config.hosts;

        // Global host-settings for rensen
        let mut settings: Settings = Settings::deserialize_yaml(&hosts)
            .map_err(|err| Trap::Deserialize(format!("Could not deserialize settings: {}", err)))?;
        
        // Removing host from the settings by extractin it's index
        for (i, host) in settings.hosts.iter().enumerate() {
            if host.hostname.to_owned() == hostname.to_owned() {
                settings.hosts.remove(i);
                break;
            }
        }

        // Writing it back to the file
        settings.serialize_yaml(&hosts)
            .map_err(|err| Trap::Serialize(format!("Could not serialize settings: {}", err)))?;

        println!("Deleted `{}`", hostname);

        Ok(())
    }

    /* mod action */

    fn mod_host(&self) -> Result<(), Trap> {
        let hosts = &self.global_config.hosts;
        let hostname = &self.operands[0];
        let style = Style::new();

        let mut settings: Settings = Settings::deserialize_yaml(&hosts)
            .map_err(|err| Trap::Deserialize(format!("Could not deserialize settings: {}", err)))?;

        // Gettings the host_config
        let host_config = match settings.associated_config(&hostname) {
            Some(config) => config,
            None => return Err(Trap::InvalidInput(format!("hostname `{}` was not found", hostname)))
        };

        println!("{}", style.clone().bold().apply_to(format!("Modifying {}, press enter to skip a field: ", hostname)));

        // Read addr
        let identifier = get_input("addr: ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?.trim().to_string();
        
        // Read Username 
        let user = get_input("user: ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?.trim().to_string();

        // Read port
        let port = get_input("port (press enter for 22): ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?.trim().to_string();

        // Read key-path
        let key = get_input("ssh-key path: ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?
            .trim().to_string();

        // Read source directory
        let source = get_input("source: ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?
            .trim().to_string();

        // Read destination/backup directory
        let destination = get_input("destination: ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?
            .trim().to_string();

        // Read backupping schedule
        let cron_schedule = get_input("backupping schedule (Cron expression): ")
            .map_err(|err| Trap::ReadInput(format!("Could not read input: {}", err)))?.trim().to_string();

        let new_host_config: HostConfig = HostConfig::from(
            match user.len() {
                0 => host_config.user.to_owned(),
                _ => user
            },
            match identifier.len() {
                0 => host_config.identifier.to_owned(),
                _ => identifier
            },
            match port.len() {
                0 => host_config.port.unwrap_or(22).to_owned(),
                _ => {
                    if port.trim().is_empty() {
                        22
                    }
                    else {
                        match port.trim().parse::<u16>() {
                            Ok(port) => port,
                            Err(err) => {
                                return Err(
                                    Trap::InvalidInput(format!("Invalid input, plese provide interger value(u16): {}", err))
                                );
                            }
                        }
                    }
                }
            },
            match key.len() {
                0 => host_config.key.unwrap_or("".into()).to_owned(),
                _ => PathBuf::from(&key), 
            },
            match source.len() {
                0 => host_config.source.to_owned(),
                _ => PathBuf::from(&source), 
            },
            match destination.len() {
                0 => host_config.destination.to_owned(),
                _ => PathBuf::from(&destination), 
            },
            match cron_schedule.len() {
                0 => host_config.cron_schedule.unwrap_or(String::from("0 0 * * *")).to_owned(),
                _ => cron_schedule
            }
        );

        println!("{}", style.clone().bold().apply_to("New config:"));
        println!("{}", new_host_config);

        // Gets the index of host.hostname == hostname.to_owned()
        for (i, host) in settings.hosts.iter().enumerate() {
            if host.hostname == hostname.to_owned() {
                settings.hosts.remove(i);
                break;
            }
        }

        // Pushes new_host to settings.hosts and serializes to path
        settings.hosts.push(Host { hostname: hostname.to_string(), config: new_host_config });
        settings.serialize_yaml(&self.global_config.hosts)
            .map_err(|err| Trap::Serialize(format!("Could not serialize settings: {}", err)))?;

        Ok(())
    }

    /* compile action */

    fn compile_snapshot(&self) -> Result<(), Trap> {
        if self.operands.len() != 1 {
            return Err(
                Trap::InvalidInput(
                    String::from("Invalid arguments for action. Use `help` for more details")
                )
            );
        }

        let hosts = &self.global_config.hosts;
        let hostname = &self.operands[0];

        let settings: Settings = Settings::deserialize_yaml(hosts)
            .map_err(|err| Trap::Deserialize(format!("Could not deserialize {:?}: {}", hosts, err)))?;

        let host_config = match settings.associated_config(&hostname) {
            Some(config) => config,
            None => return Err(Trap::InvalidInput(format!("hostname `{}` is not found", hostname)))
        };

        let mut snapshot = get_input("Snapshot: ")
            .map_err(|err| Trap::InvalidInput(format!("Could not read input: {:?}", err)))?;
        
        // Making it point to the record.json file if `latest` is given
        if snapshot.trim() == "latest" {
            snapshot = String::from("record");
        }

        let snapshot_record_path = self.global_config.backups
            .join(host_config.identifier)
            .join(".records")
            .join(format!("{}.json", snapshot.trim()));

        /* Compiling snapshot */
        let mut compiler = Compiler::from(&snapshot_record_path)?;
        compiler.compile(&self.global_config.snapshots)?;
        let _ = compiler.cleanup();

        Ok(())
    }

    /* List action */

    fn list(&self) -> Result<(), Trap> {

        // Printing hostnames of all hosts if the `list` action is pure
        if self.operands.len() == 0 {
            let settings: Settings = Settings::deserialize_yaml(&self.global_config.hosts)
                .map_err(|err| Trap::Deserialize(format!("Could not deserialize {:?}: {}", &self.global_config.hosts, err)))?;

            let style = console::Style::new();
            println!("{}", style.clone().bold().apply_to("Hosts:"));

            for host in settings.hosts {
                if host.hostname != "dummy" {
                    println!("->  {}", style.clone().bold().blue().apply_to(host.hostname));
                }
            }

            return Ok(());
        }
        else if self.operands.len() != 2 {
            return Err(
                Trap::InvalidInput(
                    String::from("Invalid arguments for action. Use `help` for more details")
                )
            );
        }

        // checking the list method, either listing `snapshots` or `config`
        let list_method = match self.operands[1].to_lowercase().as_str() {
            "snapshots" | "s" | "snap" => ListMethod::Snapshots,
               "config" | "c" | "conf" => ListMethod::Config,
            _ => return Err(Trap::InvalidInput(format!("List Method: `{}` is not recognized in this action", self.operands[0])))
        };

        match list_method {
            ListMethod::Snapshots => self.list_snapshots()?,
            ListMethod::Config => self.list_config()?,
        }

        Ok(())
    }

    fn list_config(&self) -> Result<(), Trap> {
        if self.operands.len() != 2 {
            return Err(
                Trap::InvalidInput(
                    String::from("Invalid arguments for action. Use `help` for more details")
                )
            );
        }

        let hosts = &self.global_config.hosts;
        let hostname = &self.operands[0];

        // Gettings the Settings
        let settings: Settings = Settings::deserialize_yaml(hosts)
            .map_err(|err| Trap::Deserialize(format!("Could not deserialize {:?}: {}", hosts, err)))?;

        // Extracting the config for associated hostname
        let host_config = match settings.associated_config(&hostname) {
            Some(config) => config,
            None => return Err(Trap::InvalidInput(format!("hostname `{}` was not found", hostname)))
        };

        let style = console::Style::new();
        println!("{}", style.clone().bold().apply_to(format!("Config ({}): ", hostname).as_str()));
        println!("{}", host_config);

        Ok(())
    }

    fn list_snapshots(&self) -> Result<(), Trap> {
        if self.operands.len() != 2 {
            return Err(
                Trap::InvalidInput(
                    String::from("Invalid arguments for action. Use `help` for more details")
                )
            );
        }

        let hosts = &self.global_config.hosts;
        let hostname = &self.operands[0];

        // Gettings the Settings
        let settings: Settings = Settings::deserialize_yaml(hosts)
            .map_err(|err| Trap::Deserialize(format!("Could not deserialize {:?}: {}", hosts, err)))?;

        // Extracting the config for associated hostname
        let host_config = match settings.associated_config(&hostname) {
            Some(config) => config,
            None => return Err(Trap::InvalidInput(format!("hostname `{}` was not found", hostname)))
        };

        let dir_path = self.global_config.backups
            .join(host_config.identifier)
            .join(".records");

        /* Reading directory contentens and formatting outputs */
        let entries = match fs::read_dir(&dir_path) {
            Ok(entries) => entries,
            Err(err) => return Err(Trap::FS(
                format!("Could not read directory at: `{:?}`: {}", dir_path, err)))
        };

        let mut entries_sorted_by_date: Vec<_> = entries
            .map(|entry| {
                entry.map(|e| {
                    e.metadata().and_then(|m| m.modified()).map(|t| (e, t))
                })
            })
            .filter_map(|entry| entry.ok())
            .collect();

        entries_sorted_by_date.sort_by(|a, b| a.as_ref().unwrap().1.cmp(&b.as_ref().unwrap().1));

        let style = console::Style::new();
        println!("{}", style.clone().bold().apply_to(format!("{}: ", hostname).as_str()));

        for entry in entries_sorted_by_date {
            let entry = entry.unwrap().0;
            let record = Record::deserialize_json(&entry.path())
                .map_err(|err| Trap::Deserialize(format!("Could not deserialize record, size uavailable: {}", err)))?;

            let mem_size: MemoryUsage = format_bytes(record.size);
            if let Some(file_stem) = entry.path().file_stem() {

                // Filtering out the record.json file
                if file_stem != "record" {
                    println!("->  {} {} {}", style.clone().bold().blue().apply_to(file_stem.to_str().unwrap()), mem_size.amount, mem_size.unit);
                }
            }

        }
        println!();

        Ok(())
    }

    /* run action */

    fn run_backup(&self) -> Result<(), Trap> {
        if self.operands.len() != 2 {
            return Err(
                Trap::InvalidInput(
                    String::from("Invalid arguments for action. Use `help` for more details")
                )
            );
        }

        let hosts = &self.global_config.hosts;
        let hostname = &self.operands[0];

        // Opening the settings file for all hosts
        let settings: Settings = Settings::deserialize_yaml(hosts)
            .map_err(|err| Trap::FS(format!("Could not deserialize {:?}: {}", hosts, err)))?;

        // Gettings the host config associated with hostname
        let mut host_config = match settings.associated_config(&hostname) {
            Some(config) => config,
            None => return Err(Trap::InvalidInput(format!("hostname `{}` is not found", hostname)))
        };

        // Formatting path
        let record_path = self.global_config.backups
            .join(&host_config.identifier)
            .join(".records")
            .join("record.json");

        print!("Reading record... ");
        let record = Record::deserialize_json(&record_path)
            .map_err(|err| Trap::Deserialize(format!("Could not read record {:?}: {}", record_path, err)))?;
        println!("Done");

        let mut sftp = Sftp::new(&mut host_config, &self.global_config, record, true);
        
        let backup_method: BackupMethod = match self.operands[1].to_lowercase().as_str() {
             "incremental" | "inc"| "i" => BackupMethod::Incremental,
                           "full" | "f" => BackupMethod::Full,
                                      _ => return Err(Trap::InvalidInput(String::from("Invalid input")))
        };

        if backup_method == BackupMethod::Incremental {
            sftp.incremental = true;
        }

        sftp.backup()?;

        Ok(())
    }

    /* help action */

    pub fn print_help(&self) {
        let style = Style::new();

        if self.operands.len() > 0 {
            match self.operands[0].to_lowercase().as_str() {
                "add" => {
                    println!("a, add <hostname>     Enters host-adding interface.");
                    println!(
                        "Enters the host-adding interface where you are able to specify information about\nthen host which is going to be backupped.\n\n{} Remember to have a ssh-key generated in the path you specify, and also have thepublic key on the host machine.",
                    style.bold().red().apply_to("Note:"));
                },
                "del" => {
                    println!("d, del <hostname>     Deletes host config.");
                    println!("Deletes the specified host's config from the configuration file located at probably in /etc/rensen or has specified path in /etc/rensen/rensen_config");
                },
                "mod"     => {
                    println!("m, mod <hostname>     Enters modification interface.");
                    println!("Allows you to modify a config for a host that already exists instead of readding it.");
                },
                "run"     => {
                    println!("r, run <hostname> <inc, full>   Runs backup for host based on what is specified in config."); 
                    println!("Runs the rensen backup system, either incremental or full backups. Backupped files will be stored\nat path specified in /etc/rensen/rensen_config.yml\n");
                    println!("\nAliases:\nincremental, inc, i\nfull, f");
                },
                "list"    => {
                    println!("l, list <hostname> <snapshots, config>     Lists snapshots taken of host.");
                    println!("\nsnapshots: \nThis checks the snapshots/backups taken of the host at the location specified in /etc/rensen/rensen_config.yml");
                    println!("\nconfig: \nEchos out the deserialized format of the config file, stored at location specified in /etc/rensen/rensne_config.yml");
                    println!("\nAliases: \nsnapshots, snap, s\nconfig, conf, c"); 
                },
                "compile" => {
                    println!("c, comp <hostname>     Starts compilation interface.");
                    println!("Starts the interface for compilation, where you need to specify a snapshot from what is available in `list` action.");
                },
                _ => println!("Not a regognized action"),
            }

            return;
        }

        println!("h, ?, help                             Show this info.");
        println!("q, quit, exit                          Quit ctl.");
        println!("clear                                  Clear screen.\n");

        println!("a, add <hostname>                      Enter host-adding interface.");
        println!("d, del <hostname>                      Deletes host config.");
        println!("m, mod <hostname>                      Enter modification interface.");
        println!("r, run <hostname> <inc, full>          Run backup for host based on what is specified in config."); 
        println!("l, list <hostname> <snapshots, config> list snapshots taken of host or echos config file.");
        println!("c, comp <hostname>                     Start compilation interface.");
    }
}
