pub mod rsync {
    use std::fs;
    use std::io::{self, Write, Read};
    use std::net::TcpStream;
    use ssh2::Session;
    use std::path::Path;
    use crate::logging::{log_error, ErrorType, log_info, InfoType};
    use crate::config::*;

    pub trait BackupMethod {
        fn full_backup(&mut self) -> Result<(), ErrorType>;
        fn incremental_backup(&mut self) -> Result<(), ErrorType>;
        fn connect(&mut self) -> Result<(), ErrorType>;
        fn copy_remote_directory(&self, remote_path: &Path, dest_path: &Path) -> Result<(), ErrorType>;
        fn copy_remote_file(&self, remote_path: &Path, dest_path: &Path) -> Result<(), ErrorType>;
    }

    pub struct Rsync<'a> {
        host_config: &'a HostConfig,
        // cache: Cache
        sess: Option<Session>
    }

    impl<'a> Rsync<'a> {
        pub fn new(host_config: &'a HostConfig, sess: Option<Session>) -> Self {
            Self {host_config, sess}
        }
    }

    impl BackupMethod for Rsync<'_> {

        /// Remote sync backup using ssh/sftp
        /// Default port: 22
        /// Default keypath: "$HOME/.ssh/id_rsa"
        fn full_backup(&mut self) -> Result<(), ErrorType> {

            self.connect()?;

            // key path
            let private_key_path = Path::new(self.host_config.key_path.as_ref()
                .map_or("$HOME/.ssh/id_rsa", |s| s.to_str().unwrap_or("$HOME/.ssh/id_rsa"))
            );

            // Authenticate session (private key --> public key)
            self.sess.as_ref().unwrap().userauth_pubkey_file(&self.host_config.user, None, private_key_path, None).map_err(|err| {
                log_error(ErrorType::Auth, format!("Could not Authenticate session: {}", err).as_str());
                ErrorType::Auth
            })?;

            // Copy remote path and all of it's content
            self.copy_remote_directory(&self.host_config.remote_path, &self.host_config.dest_path)?;
            println!("...copied files");
            
            Ok(())
        }

        fn incremental_backup(&mut self) -> Result<(), ErrorType> {
            // TODO: implement incremental backup.
            // * hash first 1024 bytes of file
            // * if eq, continuously hash next 1024 bytes until they are not eq.
            // * if file is eq, skip copy.
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
                let remote_file_path = remote_path.join(entryname);
                let dest_file_path = dest_path.join(&entryname);

                if stat.is_file() {
                    self.copy_remote_file(&remote_file_path, &dest_file_path)?;
                }
                else if stat.is_dir() {
                    let dest_subdir_path = dest_path.join(&entryname);
                    fs::create_dir_all(&dest_subdir_path).map_err(|err| {
                        log_error(ErrorType::FS, format!("Could not create directory: {}", err).as_str());
                        ErrorType::FS
                    })?;

                    self.copy_remote_directory(&remote_file_path, &dest_file_path)?;
                }
            }
           
            Ok(())
        }

        /// Copy remote file (remote_path) to destination (dest_path).
        fn copy_remote_file(&self, remote_path: &Path, dest_path: &Path) -> Result<(), ErrorType> {
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
