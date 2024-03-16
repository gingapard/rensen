pub mod rsync {
    use std::fs;
    use std::io::{self, Write, Read};
    use std::net::TcpStream;
    use ssh2::Session;
    use std::path::Path;
    use crate::logging::{log_error, ErrorType, log_info, InfoType};
    use crate::config::*;

    /// Remote sync backup using ssh/sftp
    /// Default port: 22
    /// Default keypath: "$HOME/.ssh/id_rsa"
    pub fn backup(host: &HostConfig) -> Result<(), ErrorType> {
        // ext ip or hostname
        let identifier = match &host.identifier {
            HostIdentifier::Ip(ip) => ip,
            HostIdentifier::Hostname(hostname) => hostname,
        };

        // ext port
        let port = host.port.unwrap_or(22);
        let mut sess = connect_ssh(&identifier, port)?;

        // key path
        let private_key_path = Path::new(host.key_path.as_ref()
            .map_or("$HOME/.ssh/id_rsa", |s| s.to_str().unwrap_or("$HOME/.ssh/id_rsa"))
        );

        // Authenticate session (private key --> public key)
        sess.userauth_pubkey_file(&host.user, None, private_key_path, None).map_err(|err| {
            log_error(ErrorType::Auth, format!("Could not Authenticate session: {}", err).as_str());
            ErrorType::Auth
        })?;

        // Copy remote path and all of it's content
        copy_remote_directory(&mut sess, &host.remote_path, &host.dest_path)?;

        println!("...copied files");
        
        Ok(())
    }

    fn connect_ssh(identifier: &str, port: u16) -> Result<Session, ErrorType> {
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

        Ok(sess)
    }

    /// Copy remote directory to destination.
    /// Will recurse and call copy_remote_file(...) until all contents are copied.
    fn copy_remote_directory(sess: &mut Session, remote_path: &Path, destination_path: &Path) -> Result<(), ErrorType> {
        // Create destination directory if it doesn't exist
        if !destination_path.exists() {
            fs::create_dir_all(destination_path).map_err(|err| {
                log_error(ErrorType::FS, format!("Could not create directory: {}", err).as_str());
                ErrorType::FS
            })?;
            println!("...destdir created");
        }
        
        let dir_entries = sess.sftp().map_err(|err| {
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
            let destination_file_path = destination_path.join(&entryname);
            println!("{:?} ---------> {:?}", remote_file_path, destination_file_path);

            if stat.is_file() {
                copy_remote_file(sess, &remote_file_path, &destination_file_path)?;
            }
            else if stat.is_dir() {
                let dest_subdir_path = destination_path.join(&entryname);
                fs::create_dir_all(&dest_subdir_path).map_err(|err| {
                    log_error(ErrorType::FS, format!("Could not create directory: {}", err).as_str());
                    ErrorType::FS
                })?;

                copy_remote_directory(sess, &remote_file_path, &destination_file_path)?;
            }
        }
       
        Ok(())
    }

    /// Copy remote file to destination.
    fn copy_remote_file(sess: &mut Session, remote_path: &Path, dest_path: &Path) -> Result<(), ErrorType> {
        let (mut channel, _) = sess.scp_recv(remote_path).map_err(|err| {
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
                // skip 
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

pub mod service_message_block {}
