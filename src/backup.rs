pub mod rsync {
    use std::fs;
    use std::io::{self, Write, Read};
    use std::net::TcpStream;
    use ssh2::{Session, FileStat};
    use std::time::SystemTime;
    use std::path::{Path, PathBuf};
    use crate::traits::{Rsync, FileSerializable};
    use crate::logging::{log_trap, Trap};
    use crate::config::*;
    use crate::utils::{make_tar_gz, set_metadata, get_datetime};
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

        pub fn debug(&self, s: &str) {
            if self.debug {
                println!("{}", s);
            }
        }

        /// Returns last_modified_time from metadata in secs (as u64)
        pub fn local_file_mtime(&self, local_file: &Path) -> Result<u64, Trap> {
            let local_metadata = fs::metadata(local_file).map_err(|err| {
                log_trap(Trap::FS, format!("Could not get metadata of local file: {}", err).as_str());
                Trap::FS
            })?;

            let local_modified = local_metadata.modified().map_err(|err| {
                log_trap(Trap::FS, format!("Could not get mod time of local file: {}", err).as_str());
                Trap::FS
            })?;

            Ok(local_modified.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs())
        }

        /// Wrapper for SFTP::stat
        pub fn remote_filestat(&self, remote_file: &Path) -> Result<FileStat, Trap> {
            let sftp = self.sess.as_ref().ok_or(Trap::FS)?.sftp().map_err(|err| {
                log_trap(Trap::FS, format!("Could not init SFTP session: {}", err).as_str());
                Trap::FS
            })?;

            let stat = sftp.stat(remote_file).map_err(|err| {
                log_trap(Trap::FS, format!("Could not get metadata of remote file: {}", err).as_str());
                Trap::FS
            })?;

            Ok(stat)
        }

        /// Returns last_modified_time for a remote file from metadata in secs (as u64)
        fn remote_file_mtime(&self, remote_file: &Path) -> Result<u64, Trap> {
            Ok(self.remote_filestat(remote_file)?.mtime.unwrap_or(u64::MAX))
        }

        /// Iterating the keys in entries and checking if they are remotly
        /// accessable still. If not, they are assumed to be deleted from the source,
        /// and therefore marked as deleted.
        fn update_deleted_entries(&mut self) -> Result<(), Trap> {
            let keys: Vec<_> = self.record.snapshot.entries.keys().cloned().collect();

            for entry in keys {
                if let Err(_) = self.remote_file_mtime(&entry) {
                    let pair = PathPair::from(
                        entry.to_path_buf(),
                        self.record.snapshot.path(&entry)
                            .unwrap()
                            .to_path_buf()
                    );

                    self.record.snapshot.mark_as_deleted(pair);
                }
            }

            Ok(())
        }

        pub fn update_entries(&mut self, base_path: &PathBuf) -> Result<(), Trap> {
            if let Ok(entries) = fs::read_dir(base_path) {
                for entry in entries {
                    let entry = match entry {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    let current_path = entry.path();

                    if current_path.is_dir() {
                        self.update_record(&current_path)?;
                    } else {

                        let source = self.local_to_source(&current_path)?;
                        let mtime = self.local_file_mtime(&current_path)?;
                        let pathpair = PathPair::from(source, current_path);

                        // If the pathpair is already marked as deleted from a previous backup
                        // (it got readded), will unmark it as deleted. Not checking mtime here
                        // as it is not relevant.
                        if self.record.snapshot.is_deleted(&pathpair) {
                            self.record.snapshot.undelete(&pathpair);
                        }

                        self.record.snapshot.add_entry(pathpair.clone(), mtime);
                    }
                }
            }

            Ok(())
        }

        pub fn update_record(&mut self, base_path: &PathBuf) -> Result<(), Trap> {
            let _ = self.update_entries(base_path)?;
            let _ = self.update_deleted_entries()?;
            Ok(())
        }

        /// Takes in a local_path, and returns it's remote path equvelent according to 'self'
        fn local_to_source(&self, current_path: &Path) -> Result<PathBuf, Trap> {
            let mut result = PathBuf::from(self.host_config.source.clone());
            let current_path_components = current_path.components().collect::<Vec<_>>(); // destination/identifier/datetime/filestem/...

            // Extracting the common prefix between current_path and self.host_config.dest_path
            // This is so that it can remove the common prefix from the current_path, and replace
            // it with self.host_config.remote_path instead
            let common_path_prefix = current_path.components()
                .zip(self.host_config.destination.components())
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a)
                .collect::<Vec<_>>()
            ;

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
        fn backup(&mut self) -> Result<(), Trap> {

            self.debug("... connecting");
            self.connect()?;
            self.debug("successs!");

            self.debug("... authenticating");
            self.auth()?;
            self.debug("successs!");

            let datetime = get_datetime();

            // Formatting destination to fit into file structure
            // Adding identifier onto dest_path, and then adding the remote_path dir onto it again.
            // Result = destination/identifier/remote_dir/ ex.
            //
            // Adding identifier: $HOME/destination/$identifier/$datetime
            self.host_config.destination = self.host_config.destination
                .join(&self.host_config.identifier)
                .join(datetime);

            let source = self.host_config.source.clone();

            // Adding filestem: $HOME/destination/identifier/$current_time/$filestem
            // This is the complete destination, where the files will be copied to.
            // The self.host_config.destination is still preserved so that it can
            // be archived and compressed later.
            let complete_destination = if let Some(stem) = &self.host_config.source.file_stem() {
                self.host_config.destination.join(stem)
            } else {
                self.host_config.destination.join(format!("{}", self.host_config.identifier))  
            };

            self.debug("... copying files");
            self.copy_remote_directory(&source, &complete_destination)?;
            self.debug("... successs!");

            self.debug("... updating records");
            self.update_record(&mut self.host_config.destination.clone())?;
            self.debug("... successs!");

            let mut record_path = self.host_config.destination.clone();

            // inner
            let _ = self.record.serialize_json(&record_path.join(".inner.json"));

            // outer
            record_path.pop();
            let _ = self.record.serialize_json(&record_path.join(".outer.json"));
                
            // compressing the ../$destination/$identifer/$datetime(parent of complete_destination)
            self.debug("... compressing");
            let _ = make_tar_gz(&self.host_config.destination, 
                Path::new(format!("{}.tar.gz", &self.host_config.destination.to_str().unwrap_or("throw")) .as_str())
            );
            self.debug("... successs!");

            Ok(())
        }

        fn auth(&mut self) -> Result<(), Trap> {

            // key path
            let default_key_path = "$HOME/.ssh/ed25519";
            let key_path = self.host_config.key_path.as_ref()
                .map(|s| s.to_str().unwrap_or(default_key_path))
                .unwrap_or(default_key_path);

            let private_key_path = Path::new(&key_path);

            println!("key_path: {:?}", private_key_path);
            println!("user: {}", self.host_config.user);
            println!("identifier: {:?}", self.host_config.identifier);

            // Authenticate session (private key --> public key)
            match self.sess.as_ref() {
                Some(session) => {
                    if let Err(err) = session.userauth_pubkey_file(&self.host_config.user, None, private_key_path, None) {
                        log_trap(Trap::Auth, format!("Could not Authenticate session: {}", err).as_str());
                        return Err(Trap::Auth);
                    }
                },
                None => {
                    log_trap(Trap::Auth, "Session is None");
                    return Err(Trap::Auth);
                }
            }

            Ok(())
        }

        fn connect(&mut self) -> Result<(), Trap> {
            let identifier = match &self.host_config.identifier {
                HostIdentifier::Ip(ip) => ip,
                HostIdentifier::Hostname(hostname) => hostname,
            };

            // ext port
            let port = self.host_config.port.unwrap_or(22);

            // Connect to SSH server
            let tcp = TcpStream::connect(format!("{}:{}", identifier, port)).map_err(|err| {
                log_trap(Trap::Connect, format!("Could not connect to host: {}", err).as_str());
                Trap::Connect
            })?;

            // Create SSH session
            let mut sess = Session::new().map_err(|err| {
                log_trap(Trap::Session, format!("Could not create SSH session: {}", err).as_str());
                Trap::Session
            })?;

            // Perform SSH handshake
            sess.set_tcp_stream(tcp);
            sess.handshake().map_err(|err| {
                log_trap(Trap::Handshake, format!("Could not perform SSH handshake: {}", err).as_str());
                Trap::Handshake
            })?;

            self.sess = Some(sess);
            Ok(())
        }
        
        /// Copy remote directory to destination.
        /// Will recurse and call copy_remote_file(...) until all contents are copied.
        fn copy_remote_directory(&self, source: &Path, destination: &Path) -> Result<(), Trap> {
            // Create destination directory if it doesn't exist
            if !destination.exists() {
                fs::create_dir_all(destination).map_err(|err| {
                    log_trap(Trap::FS, format!("Could not create directory: {}", err).as_str());
                    Trap::FS
                })?;
                println!("...destdir created");
            }
            
            let dir_entries = self.sess.as_ref().unwrap().sftp().map_err(|err| {
                log_trap(Trap::Copy, format!("Could not init SFTP: {}", err).as_str());
                Trap::Copy
            })?
            .readdir(source).map_err(|err| {
                log_trap(Trap::Copy, format!("Could not read remote directory: {}", err).as_str());
                Trap::Copy
            })?;

            for (entry, stat) in dir_entries {
                let entryname = match entry.file_name() {
                    Some(entryname) => {
                        entryname 
                    },
                    None => {
                        log_trap(Trap::Copy, "Empty file");
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
                        log_trap(Trap::FS, format!("Could not create directory: {}", err).as_str());
                        Trap::FS
                    })?;

                    self.copy_remote_directory(&new_source, &new_destination)?;
                }
            }
           
            Ok(())
        }

        /// Copy remote file (source) to destination.
        fn copy_remote_file(&self, source: &Path, destination: &Path) -> Result<(), Trap> {
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
                log_trap(Trap::Copy, format!("Could not receive file from remote path: {}", err).as_str());
                Trap::Copy
            })?;

            let mut file = fs::File::create(destination).map_err(|err| {
                log_trap(Trap::FS, format!("Could not create file: {}", err).as_str());
                Trap::FS
            })?;

            let mut buffer = [0; 4096];
            loop {
                match channel.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        file.write_all(&buffer[..n]).map_err(|err| {
                            log_trap(Trap::FS, format!("Could not write to file: {}", err).as_str());
                            Trap::FS
                        })?;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(err) => {
                        log_trap(Trap::Copy, format!("Could not read from channel: {}", err).as_str());
                        return Err(Trap::Copy);
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
