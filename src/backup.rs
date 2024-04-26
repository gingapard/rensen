pub mod rsync {
    use std::fs;
    use std::io::{self, Write, Read};
    use std::net::TcpStream;
    use ssh2::{Session, FileStat};
    use std::time::SystemTime;
    use std::path::{Path, PathBuf};
    use crate::traits::{Rsync, FileSerializable};
    use crate::logging::{log_error, ErrorType};
    use crate::config::*;
    use crate::utils::{archive_compress_dir, set_metadata, get_datetime};
    use crate::record::Record;
    use crate::snapshot::PathPair;

    pub struct Sftp<'a> {
        pub host_config: &'a mut HostConfig,
        pub record: Record,
        pub sess: Option<Session>,
        pub incremental: bool,
        pub debug: bool,
    }

    impl<'a> Sftp<'a> {
        pub fn new(host_config: &'a mut HostConfig, record: Record, debug: bool) -> Self {
            Self {
                host_config,
                record,
                sess: None,
                incremental: false,
                debug,
            }
        }
        
        /// Returns last_modified_time from metadata in secs (as u64)
        pub fn local_file_mtime(&self, local_file: &Path) -> Result<u64, ErrorType> {
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

        /// Wrapper for SFTP::stat
        pub fn remote_filestat(&self, remote_file: &Path) -> Result<FileStat, ErrorType> {
            let sftp = self.sess.as_ref().ok_or(ErrorType::FS)?.sftp().map_err(|err| {
                log_error(ErrorType::FS, format!("Could not init SFTP session: {}", err).as_str());
                ErrorType::FS
            })?;

            let stat = sftp.stat(remote_file).map_err(|err| {
                log_error(ErrorType::FS, format!("Could not get metadata of remote file: {}", err).as_str());
                ErrorType::FS
            })?;

            Ok(stat)
        }

        /// Returns last_modified_time for a remote file from metadata in secs (as u64)
        fn remote_file_mtime(&self, remote_file: &Path) -> Result<u64, ErrorType> {
            Ok(self.remote_filestat(remote_file)?.mtime.unwrap_or(u64::MAX))
        }

        pub fn update_record(&mut self, base_path: &PathBuf) -> Result<(), ErrorType> {
            let mut to_remove: Vec<PathPair> = Vec::new();
            let mut current_files: Vec<PathPair> = Vec::new();

            if let Ok(entries) = fs::read_dir(base_path) {
                for entry in entries {
                    let entry = entry.unwrap();
                    let current_path = entry.path();

                    if current_path.is_dir() {
                        self.update_record(&current_path)?;
                    } else {
                        let source = self.local_to_source(&current_path)?;
                        let mtime = self.local_file_mtime(&current_path)?;
                        println!("mtime: {}", mtime);

                        let pathpair = PathPair::from(source, current_path);

                        // Because deleted files are added to the deleted vector when they are
                        // deleted, it is checking if it got re-added, and can therefore be removed
                        // from deleted_entries
                        if self.record.snapshot.is_deleted(&pathpair) {
                            self.record.snapshot.undelete(&pathpair);
                        }

                        self.record.snapshot.add_entry(pathpair.clone(), mtime);
                        current_files.push(pathpair);
                    }
                }
            }

            // With this, it is checking if any of the keys from the record (previous iteration)
            // Are missing from this iterations. If so, they are sentenced to be removed from
            // the record to keep track of deleted files.
            
            println!("Current files: {:?}", current_files);
            let sources: Vec<PathBuf> = current_files.iter().map(|pair| pair.source.clone()).collect();
            for entry in self.record.snapshot.entries.keys() {
                let mut found = false;
                for source in sources.iter() {
                    if entry == source {
                        found = true;
                        break;
                    }
                }

                if !found {
                    if let Some(pair) = current_files.iter().find(|pair| pair.source == *entry) {
                        println!("Deleted: {:?}", pair);
                        to_remove.push(pair.clone());
                    }
                }
            }

            // Removing the entries that got deleted.
            println!("To remove: {:?}", to_remove);
            for entry in to_remove {
                self.record.snapshot.mark_as_deleted(entry);
            }

            Ok(())
        }

        /// Takes in a local_path, and returns it's remote path equvelent according to 'self'
        fn local_to_source(&self, current_path: &Path) -> Result<PathBuf, ErrorType> {
            let mut result = PathBuf::from(self.host_config.source.clone());
            let current_path_components = current_path.components().collect::<Vec<_>>(); // destination/identifier/datetime/filestem/...

            // Extracting the common prefix between current_path and self.host_config.dest_path
            // This is so that it can remove the common prefix from the current_path, and replace
            // it with self.host_config.remote_path instead
            let common_path_prefix = current_path.components()
                .zip(self.host_config.destination.components())
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a)
                .collect::<Vec<_>>();

            let ramaining_components = current_path_components.iter().skip(common_path_prefix.len() + 1);
            for component in ramaining_components {
                result.push(component);
            }

            Ok(result)
        }


    }

    impl Rsync for Sftp<'_> {

        /// Remote sync backup using ssh/sftp
        /// Default port: 22
        /// Default keypath: "$HOME/.ssh/id_rsa"
        /// Compare last-modified timestamp of files with matching namesm,
        /// ignoring those with matching timestamp. 
        /// You take one full backup, and the take incremental backups 
        /// the next days. Put a setting to take a new *full* backup every week or so.
        /// Backups older than a specific amount (maybe 30 days) will be deleted.
        /// 
        /// ***File structure example***
        ///
        /// 192.168.1.220
        ///     | record.json
        ///     | 2023-01-11_12-34-56.tar.gz
        ///         | 'remote_path_stem/'
        ///     | 2023-01-12_12-34-56.tar.gz
        ///         | 'remote_path_stem/'
        ///     | ...tar.gz
        ///
        ///
        /// *record.json*
        /// 
        /// path: mtime as u64,
        /// ...
        ///
        ///
        fn backup(&mut self) -> Result<(), ErrorType> {
            self.connect()?;
            self.auth()?;

            // Formatting destination to fit into file structure
            // Adding identifier onto dest_path, and then adding the remote_path dir onto it again.
            // Result = destination/identifier/remote_dir/ ex.
            //
            // Adding identifier: $HOME/destination/$identifier
            self.host_config.destination = self.host_config.destination.join(&self.host_config.identifier);
            // Adding current_time: $HOME/destination/$identifier/$current_time
            self.host_config.destination = self.host_config.destination.join(get_datetime());

            // Adding filestem: $HOME/destination/identifier/$current_time/$filestem
            // This is the complete destination, where the files will be copied to.
            // The self.host_config.destination is still preserved so that it can
            // be archived and compressed later.
            let complete_destination = if let Some(stem) = &self.host_config.source.file_stem() {
                self.host_config.destination.join(stem)
            } else {
                self.host_config.destination.join(format!("{}", self.host_config.identifier))  
            };

            // Copy remote path and all of it's content
            let source = self.host_config.source.clone();
            self.copy_remote_directory(&source, &complete_destination)?;
            // update records
            self.update_record(&mut self.host_config.destination.clone())?;
            /* self.record.intervals.push(complete_destination.to_path_buf()); */
            println!("{}", self.record);

            // Ensure "record.json" is put in with the backupped files' root folder
            // ($HOME/destination/identifier/record.json)
            let mut record_path = self.host_config.destination.clone();
            record_path.pop();

            // adding .record.json record to root dir as wall as to the root of 
            // the backupped files xxx/destination/../.record.json
            let _ = self.record.serialize_json(&record_path.join(".record.json"));
            let _ = self.record.serialize_json(&self.host_config.destination.join(".record.json"));
                
            let _ = archive_compress_dir(&self.host_config.destination, 
                Path::new(format!("{}.tar.gz", &self.host_config.destination.to_str().unwrap_or("throw")) .as_str())
            );
            
            println!("... copied files");
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
        fn copy_remote_directory(&self, source: &Path, destination: &Path) -> Result<(), ErrorType> {
            // Create destination directory if it doesn't exist
            if !destination.exists() {
                fs::create_dir_all(destination).map_err(|err| {
                    log_error(ErrorType::FS, format!("Could not create directory: {}", err).as_str());
                    ErrorType::FS
                })?;
                println!("...destdir created");
            }
            
            let dir_entries = self.sess.as_ref().unwrap().sftp().map_err(|err| {
                log_error(ErrorType::Copy, format!("Could not init SFTP: {}", err).as_str());
                ErrorType::Copy
            })?
            .readdir(source).map_err(|err| {
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
                let new_source = source.join(entryname);
                let new_destination = destination.join(entryname);

                if stat.is_file() {
                    self.copy_remote_file(&new_source, &new_destination)?;
                }
                else if stat.is_dir() {
                    let destination_subdir = destination.join(&entryname);
                    fs::create_dir_all(&destination_subdir).map_err(|err| {
                        log_error(ErrorType::FS, format!("Could not create directory: {}", err).as_str());
                        ErrorType::FS
                    })?;

                    self.copy_remote_directory(&new_source, &new_destination)?;
                }
            }
           
            Ok(())
        }

        /// Copy remote file (source) to destination.
        fn copy_remote_file(&self, source: &Path, destination: &Path) -> Result<(), ErrorType> {
            if self.incremental {
                // check mtime data at local and source
                let remote_mtime: &u64 = &self.remote_file_mtime(source)?; 

                let dest_as_source = self.local_to_source(destination)?;
                if remote_mtime <= self.record.snapshot.mtime(&dest_as_source).unwrap_or(&0) {
                    println!("not copying");
                    return Ok(());
                }
            }

           /*---------------------------------------------------------------------------*
            * Starting proceess of copying the file from remote to locally, also ensuring*
            * metadata and permissons of the the file.                                  *
            *---------------------------------------------------------------------------*/

            let (mut channel, _) = self.sess.as_ref().unwrap().scp_recv(source).map_err(|err| {
                log_error(ErrorType::Copy, format!("Could not receive file from remote path: {}", err).as_str());
                ErrorType::Copy
            })?;

            let mut file = fs::File::create(destination).map_err(|err| {
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

            // Sets metadata for the newly created file to the same as the remote file.
            let stat = self.remote_filestat(source)?;
            let _ = set_metadata(&mut file, stat);

            let m_data = file.metadata();
            println!("{:?}", m_data.unwrap().modified());

            Ok(())
        }
    }

    pub struct Samba {}
}
