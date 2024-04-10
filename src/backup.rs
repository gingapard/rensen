pub mod rsync {
    use std::fs;
    use std::io::{self, Write, Read};
    use std::net::TcpStream;
    use ssh2::{Session, FileStat};
    use std::time::SystemTime;
    use std::path::{Path, PathBuf};
    use crate::traits::{BackupMethod, FileSerializable};
    use crate::logging::{log_error, ErrorType, log_info, InfoType};
    use crate::config::*;
    use crate::utils::archive_compress_dir;
    use crate::record::Record;

    pub struct Rsync<'a> {
        pub host_config: &'a mut HostConfig,
        pub record: Record,
        pub sess: Option<Session>,
        pub index: usize
    }

    impl<'a> Rsync<'a> {
        pub fn new(host_config: &'a mut HostConfig, record: Record) -> Self {
            Self {
                host_config,
                record,
                sess: None,
                index: 0
            }
        }

        /// Returns last_modified_time from metadata in secs (as u64)
        pub fn local_file_modified_time(&self, local_file: &PathBuf) -> Result<u64, ErrorType> {
            let local_metadata = fs::metadata(local_file).map_err(|err| {
                log_error(ErrorType::FS, format!("Could not get metadata of local file: {}", err).as_str());
                ErrorType::FS
            })?;

            let local_modified = local_metadata.modified().map_err(|err| {
                log_error(ErrorType::FS, format!("Could not get mod time of local file: {}", err).as_str());
                ErrorType::FS
            })?;

            Ok(local_modified.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs())
        }

        /// Returns last_modified_time for a remote file from metadata in secs (as u64)
        pub fn remote_file_modified_time(&self, remote_file: &Path) -> Result<u64, ErrorType> {
            let sftp = self.sess.as_ref().ok_or(ErrorType::FS)?.sftp().map_err(|err| {
                log_error(ErrorType::FS, format!("Could not init SFTP session: {}", err).as_str());
                ErrorType::FS
            })?;

            let remote_metadata = sftp.stat(remote_file).map_err(|err| {
                log_error(ErrorType::FS, format!("Could not get metadata of remote file: {}", err).as_str());
                ErrorType::FS
            })?;

            Ok(remote_metadata.mtime.unwrap_or(0))
        }

        fn recurs_update_record(&mut self, base_path: &PathBuf) -> Result<(), ErrorType> {
            if let Ok(entries) = fs::read_dir(base_path) {
                for entry in entries {
                    let entry = entry.unwrap();
                    let current_path = entry.path();

                    if current_path.is_dir() {
                        self.recurs_update_record(&current_path)?;
                    }
                    else {
                        self.record.entries.insert(current_path.clone(), self.local_file_modified_time(&current_path)?);
                    }
                }
            }

            Ok(())
        }
    }

    impl BackupMethod for Rsync<'_> {

        /// Remote sync backup using ssh/sftp
        /// Default port: 22
        /// Default keypath: "/home/$HOME/.ssh/id_rsa"
        fn full_backup(&mut self) -> Result<(), ErrorType> {
            self.connect()?;
            self.auth()?;

            /* Formatting dest_path to fit into file structure
             * Adding identifier onto dest_path, and then adding the remote_path dir onto it again.
             * Result = dest_path/identifier/remote_dir/ ex.
             * /home/user/backups/192.168.1.1/backupped_files
             */
            self.host_config.dest_path = self.host_config.dest_path.join(&self.host_config.identifier);
            self.host_config.dest_path = if let Some(stem) = self.host_config.remote_path.file_stem() {
                self.host_config.dest_path.join(stem)
            } else {
                self.host_config.dest_path.join(format!("{}", self.host_config.identifier))  
            };

            // Copy remote path and all of it's content
            self.copy_remote_directory(&self.host_config.remote_path, &self.host_config.dest_path)?;

            // update records
            self.record.entries.clear();
            self.recurs_update_record(&mut self.host_config.dest_path.clone())?;
            let _ = self.record.serialize_json(Path::new("record.json"));

            let _ = archive_compress_dir(&self.host_config.dest_path, 
                Path::new(format!("{}.tar.gz", &self.host_config.dest_path.to_str().unwrap_or("throw")) .as_str())
            );
            
            println!("... copied files");
            Ok(())
        }

        /// Compare last-modified timestamp of files with matching namesm,
        /// ignoring those with matching timestamp. 
        /// You take one full backup, and the take incremental backups 
        /// the next days. Put a setting to take a new *full* backup every week or so.
        /// Backups older than a specific amount (maybe 30 days) will be deleted.
        /// 
        /// ***File structure***
        ///
        /// 192.168.x.x
        ///     | record.yaml
        ///     | first_full_backup.tar.gz
        ///     | next_incremental_backup.tar.gz
        ///     | ...tar.gz
        ///
        ///
        fn incremental_backup(&mut self) -> Result<(), ErrorType> {
            self.connect()?;
            self.auth()?;

            Ok(())
        }

        fn auth(&mut self) -> Result<(), ErrorType> {
            // key path
            let private_key_path = Path::new(self.host_config.key_path.as_ref()
                .map_or("$HOME/.ssh/id_rsa", |s| s.to_str().unwrap_or("/home/$HOME/.ssh/id_rsa"))
            );
        
            println!("key_path: {:?}", private_key_path);
            println!("user: {}", self.host_config.user);
            println!("identifier: {:?}", self.host_config.identifier);

            // Authenticate session (private key --> public key)
           match self.sess.as_ref() {
                Some(session) => {
                    if let Err(err) = session.userauth_pubkey_file(&self.host_config.user, None, private_key_path, None) {
                        log_error(ErrorType::Auth, format!("Could not Authenticate session: {}", err).as_str());
                        return Err(ErrorType::Auth);
                    }
                },
                None => {
                    log_error(ErrorType::Auth, "Session is None");
                    return Err(ErrorType::Auth);
                }
            }

            println!("... auth");

            Ok(())
        }

        fn connect(&mut self) -> Result<(), ErrorType> {
            let identifier = match &self.host_config.identifier {
                HostIdentifier::Ip(ip) => ip,
                HostIdentifier::Hostname(hostname) => hostname,
            };

            // ext port
            let port = self.host_config.port.unwrap_or(22);

            // Connect to SSH server
            let tcp = TcpStream::connect(format!("{}:{}", identifier, port)).map_err(|err| {
                log_error(ErrorType::Connect, format!("Could not connect to host: {}", err).as_str());
                ErrorType::Connect
            })?;

            // Create SSH session
            let mut sess = Session::new().map_err(|err| {
                log_error(ErrorType::Session, format!("Could not create SSH session: {}", err).as_str());
                ErrorType::Session
            })?;

            // Perform SSH handshake
            sess.set_tcp_stream(tcp);
            sess.handshake().map_err(|err| {
                log_error(ErrorType::Handshake, format!("Could not perform SSH handshake: {}", err).as_str());
                ErrorType::Handshake
            })?;

            self.sess = Some(sess);
            Ok(())
        }
        
        /// Copy remote directory to destination.
        /// Will recurse and call copy_remote_file(...) until all contents are copied.
        fn copy_remote_directory(&self, remote_path: &Path, dest_path: &Path) -> Result<(), ErrorType> {
            // Create destination directory if it doesn't exist
            if !dest_path.exists() {
                fs::create_dir_all(dest_path).map_err(|err| {
                    log_error(ErrorType::FS, format!("Could not create directory: {}", err).as_str());
                    ErrorType::FS
                })?;
                println!("...destdir created");
            }
            
            let dir_entries = self.sess.as_ref().unwrap().sftp().map_err(|err| {
                log_error(ErrorType::Copy, format!("Could not init SFTP: {}", err).as_str());
                ErrorType::Copy
            })?
            .readdir(remote_path).map_err(|err| {
                log_error(ErrorType::Copy, format!("Could not read remote directory: {}", err).as_str());
                ErrorType::Copy
            })?;

            for (entry, stat) in dir_entries {
                let entryname = match entry.file_name() {
                    Some(entryname) => {
                        entryname 
                    },
                    None => {
                        log_error(ErrorType::Copy, "Empty file");
                        continue;
                    },
                };

                // format paths
                let new_remote_path = remote_path.join(entryname);
                let new_dest_path = dest_path.join(entryname);

                if stat.is_file() {
                    self.copy_remote_file(&new_remote_path, &new_dest_path)?;
                }
                else if stat.is_dir() {
                    let dest_subdir_path = dest_path.join(&entryname);
                    fs::create_dir_all(&dest_subdir_path).map_err(|err| {
                        log_error(ErrorType::FS, format!("Could not create directory: {}", err).as_str());
                        ErrorType::FS
                    })?;

                    self.copy_remote_directory(&new_remote_path, &new_dest_path)?;
                }
            }
           
            Ok(())
        }

        /// Copy remote file (remote_path) to destination (dest_path).
        fn copy_remote_file(&self, remote_path: &Path, dest_path: &Path) -> Result<(), ErrorType> {
            let test_var = &self.remote_file_modified_time(&remote_path).unwrap();
            println!("{}", test_var);

            let (mut channel, _) = self.sess.as_ref().unwrap().scp_recv(remote_path).map_err(|err| {
                log_error(ErrorType::Copy, format!("Could not receive file from remote path: {}", err).as_str());
                ErrorType::Copy
            })?;

            let mut file = fs::File::create(dest_path).map_err(|err| {
                log_error(ErrorType::FS, format!("Could not create file: {}", err).as_str());
                ErrorType::FS
            })?;

            let mut buffer = [0; 4096];
            loop {
                match channel.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        file.write_all(&buffer[..n]).map_err(|err| {
                            log_error(ErrorType::FS, format!("Could not write to file: {}", err).as_str());
                            ErrorType::FS
                        })?;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(err) => {
                        log_error(ErrorType::Copy, format!("Could not read from channel: {}", err).as_str());
                        return Err(ErrorType::Copy);
                    }
                }
            }

            Ok(())
        }
    }
}

pub mod service_message_block {}
