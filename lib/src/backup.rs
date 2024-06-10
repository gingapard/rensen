pub mod rsync {
    use std::fs;
    use std::io::{self, stdout, Write, Read};
    use std::net::TcpStream;
    use ssh2::{Session, FileStat};
    use std::time::SystemTime;
    use std::path::{Path, PathBuf}; 
    use std::ffi::OsStr;
    use console::Style;
    use std::rc::Rc;

    use crate::traits::*;
    use crate::logging::Trap;
    use crate::config::*;
    use crate::utils::{make_tar_gz, set_metadata, get_datetime, get_file_sz};
    use crate::record::Record;
    use crate::snapshot::{PathPair, FileEntry, Snapshot};

    pub struct Sftp<'a> {
        
        /* Public */
        pub host_config: &'a HostConfig,
        pub global_config: &'a GlobalConfig,
        pub record: Record,
        pub sess: Option<Session>,
        pub incremental: bool,
        pub debug: bool,

        /* Private */
        host_root_path: Option<PathBuf>,
        snapshot_root_path: Option<PathBuf>,
        complete_destination: Option<PathBuf>,
        style: Rc<Style>,
    }

    impl<'a> Sftp<'a> {
        pub fn new(host_config: &'a HostConfig, global_config: &'a GlobalConfig, record: Record, debug: bool) -> Self {
            Self {
                host_config,
                global_config,
                record,
                sess: None,
                incremental: false,
                debug,

                host_root_path: None,
                snapshot_root_path: None,
                complete_destination: None,
                style: Rc::new(Style::new()),
            }
        }

        pub fn debug(&self, s: &str) -> Result<(), Trap> {
            if self.debug == true {
                print!("{}", s);
                io::stdout()
                    .flush()
                    .map_err(|err| {
                        Trap::STD(format!("Coult not flush output: {}", err))
                    })?;
            }

            Ok(())
        }

        /// Returns last_modified_time from metadata in secs (as u64)
        pub fn local_file_mtime(&self, local_file: &Path) -> Result<u64, Trap> {
            let local_metadata = fs::metadata(local_file).map_err(|err| {
                Trap::Metadata(
                    format!("Could not get metadata of local file: {}.\nMay be missing or corrupt!", err))
            })?;

            let local_modified = local_metadata.modified().map_err(|err| {
                Trap::Metadata(format!("Could not get mod time of local file: {}", err))
            })?;

            Ok(local_modified.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs())
        }

        /// Wrapper for SFTP::stat
        pub fn remote_filestat(&self, remote_file: &Path) -> Result<FileStat, Trap> {
            let sftp = self.sess.as_ref().ok_or(Trap::FS(String::from("Session unavailable")))?.sftp().map_err(|err| {
                Trap::Session(format!("Could not init SFTP session: {}", err))
            })?;

            let stat = sftp.stat(remote_file).map_err(|err| {
                Trap::Metadata(
                    format!("Could not get metadata of remote file: {}\nMay be missing or corrupt!", err))
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

                    // println!("Deleting: {:?}", pair);
                    self.record.snapshot.mark_as_deleted(pair);
                }
            }

            Ok(())
        }

        pub fn update_entries(&mut self, dir_path: &PathBuf /*, snapshot: &mut Snapshot */) -> Result<(), Trap> {
            
            // let snapshot_root_path = Rc::from(self.snapshot_root_path.clone().unwrap());
            let snapshot_root_path = self.snapshot_root_path.clone().unwrap();

            if let Ok(entries) = fs::read_dir(dir_path) {
                for entry in entries {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(_) => continue,
                    };

                    let current_path = entry.path();

                    if current_path.is_dir() {
                        self.update_entries(&current_path)?;
                    } else {

                        // TODO: MULTITHREADING
                        let source = self.into_source(&current_path)?; 
                        let mtime = self.local_file_mtime(&current_path)?; 
                        let size = get_file_sz(&current_path);
                        let _ = self.debug(format!("{} {:?}... ", <Style as Clone>::clone(&self.style).bold().cyan().apply_to(String::from("Recording")), &current_path).as_str());

                        // If the pathpair is already marked as deleted from a previous backup
                        // (it got readded), will unmark it as deleted. Not checking mtime here
                        // as it is not relevant.

                        let pathpair = PathPair::from(source.clone(), current_path.clone());
                        if self.record.snapshot.is_deleted(&pathpair) {
                            self.record.snapshot.undelete(&pathpair);
                        }

                        self.record.snapshot.entries.insert(source, FileEntry { file_path: current_path, snapshot_path: snapshot_root_path.clone(), mtime, size});
                        let _ = self.debug("Done\n");
                    }
                }
            }

            Ok(())
        }

        /// base_path: path to dir the files were copied to.
        pub fn update_record(&mut self, base_path: &PathBuf) -> Result<(), Trap> {
            // let mut snapshot = Snapshot::new();

            let _ = self.update_entries(base_path)?;
            let _ = self.update_deleted_entries()?;

            // Count up total size
            let mut total_size = 0;
            for key in self.record.snapshot.entries.keys() {
                if let Some(size) = self.record.snapshot.size(key) {
                    total_size += size;
                }
            }

            self.record.size = total_size;

            Ok(())
        }

        /// Takes in a local_path, and returns it's remote path equvelent according to 'self'
        fn into_source(&self, current_path: &Path) -> Result<PathBuf, Trap> {
            let mut result = PathBuf::from(self.host_config.source.clone());
            let current_path_components = current_path.components().collect::<Vec<_>>(); 

            // Extracting the common prefix between current_path and self.host_config.dest_path
            // This is so that it can remove the common prefix from the current_path, and replace
            // it with self.host_config.remote_path instead
            let common_path_prefix = current_path.components()
                .zip(self.snapshot_root_path.clone().unwrap().components())
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
        /// Default keypath: "$HOME/.ssh/ed25519
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

            let _ = self.debug("Connecting to host... ")?;
            self.connect()?;
            let _ = self.debug("Done\n")?;

            let _ = self.debug("Authenticating... ")?;
            self.auth()?;
            let _ = self.debug("Done\n")?;

            let datetime = get_datetime();
            let source = &self.host_config.source;

            // $HOME/destination/$identifier
            self.host_root_path = Some(self.global_config.backups
                .join(&self.host_config.identifier));

            // $HOME/destination/$identifier/$datetime
            self.snapshot_root_path = Some(self.host_root_path.clone().unwrap()
                .join(datetime));

            // $HOME/destination/$identifier/$datetime/dir_name
            self.complete_destination = if let Some(stem) = &self.host_config.source.file_stem() {
                Some(self.snapshot_root_path.clone().unwrap().join(stem))
            } else {
                Some(self.snapshot_root_path.clone().unwrap().join(format!("{}", self.host_config.identifier)))
            };

            self.copy_remote_directory(&source, &self.complete_destination.clone().unwrap())?;

            self.update_record(&mut self.snapshot_root_path.clone().unwrap())?;
            let _ = self.debug("Done\n")?;

            // $HOME/destination/$identifier/.records
            let record_dir_path = self.host_root_path.clone().unwrap()
                .join(".records");

            if !record_dir_path.exists() {
                fs::create_dir_all(&record_dir_path).map_err(|err| {
                    Trap::FS(format!("Could not create directory: {}", err))
                })?;
            }

            // Serializeing records
            let _ = self.debug("Wrting records... ")?;
            let _ = self.record.serialize_json(&record_dir_path.join("record.json"));
            println!("Done");

            let snapshot_root_path_binding = self.snapshot_root_path.clone().unwrap();
            let snapshot_root_file_stem = match snapshot_root_path_binding.file_name() {
                Some(stem) => stem,
                _ => &OsStr::new("broken")
            };

            let _ = self.record.serialize_json(&record_dir_path.join(
                format!("{}.json", snapshot_root_file_stem.to_str().unwrap_or("broken"))
            ));

            // Compressing and archive
            let archive_compress_dest: &str = snapshot_root_path_binding.to_str().unwrap();

            let _ = make_tar_gz(
                self.snapshot_root_path.clone().unwrap(),
                format!("{}.tar.gz", archive_compress_dest)
            );

            let _ = self.debug("Status: OK\n")?;
            
            Ok(())
        }

        fn auth(&mut self) -> Result<(), Trap> {

            // key path
            let default_key_path = "$HOME/.ssh/ed25519";
            let key_path = self.host_config.key.as_ref()
                .map(|s| s.to_str().unwrap_or(default_key_path))
                .unwrap_or(default_key_path);

            let private_key_path = Path::new(&key_path);

            // Authenticate session (private key --> public key)
            match self.sess.as_ref() {
                Some(session) => {
                    if let Err(err) = session.userauth_pubkey_file(&self.host_config.user, None, private_key_path, None) {
                        return Err(Trap::Auth(
                                format!("Could not Authenticate session: {}\nMake sur ethe ssh-key is at hosts specified key-path", err)
                                )
                        );
                    }
                },
                None => {
                    return Err(Trap::Auth(String::from("Sessions is None")));
                }
            }

            Ok(())
        }

        fn connect(&mut self) -> Result<(), Trap> {
            let identifier = &self.host_config.identifier;
            let port = self.host_config.port.unwrap_or(22);

            // Connect to SSH server
            let tcp = TcpStream::connect(format!("{}:{}", identifier, port)).map_err(|err| {
                Trap::Connect(format!("Could not connect to host: {}\nHost unreachable!", err))

            })?;

            // Create SSH session
            let mut sess = Session::new().map_err(|err| {
                Trap::Session(format!("Could not create SSH session: {}", err))

            })?;

            // Perform SSH handshake
            sess.set_tcp_stream(tcp);
            sess.handshake().map_err(|err| {
                Trap::Handshake(format!("Could not perform SSH handshake: {}", err))
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
                    Trap::FS(format!("Could not create directory: {}", err))

                })?;
            }
            
            let dir_entries = self.sess.as_ref().unwrap().sftp().map_err(|err| {
                Trap::Copy(format!("Could not init SFTP: {}", err))

            })?
            .readdir(source).map_err(|err| {
                Trap::Copy(format!("Could not read remote directory: {}", err))

            })?;

            for (entry, stat) in dir_entries {
                let entryname = match entry.file_name() {
                    Some(entryname) => {
                        entryname 
                    },
                    None => {
                        continue;
                    },
                };

                // format paths
                let new_source = source.join(entryname);
                let new_destination = destination.join(entryname);

                if stat.is_file() {
                    match self.copy_remote_file(&new_source, &new_destination) {
                        Ok(_) => (),
                        Err(err) => { 
                            println!("{} Could not receive file, please check permissions: {:?}", <Style as Clone>::clone(&self.style).bold().red().apply_to(String::from("Skipping")), err);
                        }
                    }
                }
                else if stat.is_dir() {
                    let destination_subdir = destination.join(&entryname);
                    fs::create_dir_all(&destination_subdir).map_err(|err| {
                        Trap::FS(format!("Could not create directory: {}\nCheck permissions!", err))
                    })?;

                    match self.copy_remote_directory(&new_source, &new_destination) {
                        Ok(_) => (),
                        Err(err) => { 
                            println!("{} Directory out of reach, please check permissions: {:?}", <Style as Clone>::clone(&self.style).bold().red().apply_to(String::from("Skipping")), err);
                        }
                    }
                }
            }
           
            Ok(())
        }

        /// Copy remote file (source) to destination.
        fn copy_remote_file(&self, source: &Path, destination: &Path) -> Result<(), Trap> {
            // TODO: MULTITHREADING
            
            if self.incremental {
                // check mtime data at local and source
                let remote_mtime: &u64 = &self.remote_file_mtime(source)?; 

                let dest_as_source = self.into_source(destination)?;
                if remote_mtime <= self.record.snapshot.mtime(&dest_as_source).unwrap_or(&0) {
                    println!("{} {}@{}:{:?}", <Style as Clone>::clone(&self.style).bold().blue().apply_to(String::from("Skipping")), self.host_config.user, self.host_config.identifier, source);
                    return Ok(());
                }
            }

           /*---------------------------------------------------------------------------*
            * Starting proceess of copying the file from remote to locally, also ensuring*
            * metadata and permissons of the the file.                                  *
            * Need to be run in sudo if it is going to write in /
            *---------------------------------------------------------------------------*/

            let (mut channel, _) = self.sess.as_ref().unwrap().scp_recv(source).map_err(|err| {
                Trap::Copy(format!("Could not receive file from remote path: {}", err))
            })?;


            let mut file = fs::File::create(destination).map_err(|err| {
                Trap::FS(format!("Could not create file: {}\nCheck permissions!", err))
            })?;

            print!("{} {}@{}:{:?} ... ", <Style as Clone>::clone(&self.style).bold().blue().apply_to(String::from("Getting")), self.host_config.user, self.host_config.identifier, source);
            let mut buffer = [0; 4096];
            loop {
                match channel.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        file.write_all(&buffer[..n]).map_err(|err| {
                            Trap::FS(format!("Could not write to file: {}", err))
                        })?;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(err) => {
                        return Err(Trap::Channel(format!("Could not read from channel: {}", err)));
                    }
                }
            }
            println!("Done");

            // Sets metadata for the newly created file to the same as the remote file.
            // print!("Copying metadata... ");            
            let stat = self.remote_filestat(source)?;
            let _ = set_metadata(&mut file, stat);
            // println!("Done");

            Ok(())
        }
    }

    pub struct Samba {}
}
