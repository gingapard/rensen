use std::fs::{self, File};
use std::io::{self, Write, Read, Error as IOError};
use std::net::TcpStream;
use ssh2::{Session, Error as SSHError};
use std::path::{Path, PathBuf};
use crate::logging::{log_error, ErrorType, log_info, InfoType};
use crate::serde;

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
    println!("{:?}", dir_entries);

    for (entry, _) in dir_entries {
        let filename = match entry.file_name() {
            Some(filename) => {
                println!("{:?}", filename);
                filename
            },
            None => {
                log_error(ErrorType::Copy, "empty file");
                continue;
            },
        };

        // format paths
        let remote_file_path = remote_path.join(filename);
        let destination_file_path = destination_path.join(&filename);

        if entry.is_file() {
            copy_remote_file(sess, &remote_file_path, &destination_file_path)?;
        }
        else if entry.is_dir() {
            copy_remote_directory(sess, &remote_file_path, &destination_file_path)?;
        }
    }
   
    Ok(())
}

// Use in recursing directory
fn copy_remote_file(sess: &mut Session, remote_path: &Path, destination_path: &Path) -> Result<(), ErrorType> {
    let (mut channel, _) = sess.scp_recv(remote_path).map_err(|err| {
        log_error(ErrorType::Copy, format!("Could not receive file from remote path: {}", err).as_str());
        ErrorType::Copy
    })?;

    let mut file = fs::File::create(destination_path).map_err(|err| {
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
            }
        }
    }

    Ok(())
}

/// Remote sync backup using ssh/sftp
/// Default port: 22
/// Default keypaht: "$HOME/.ssh/id_rsa"
pub fn backup_rsync(host: &serde::Host) -> Result<(), ErrorType> {
    // ext ip or hostname
    let identifier = match &host.identifier {
        serde::HostIdentifier::Ip(ip) => ip,
        serde::HostIdentifier::Hostname(hostname) => hostname,
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
    copy_remote_directory(&mut sess, &host.remote_path, &host.destination_path)?;
    println!("...copied files");
    
    Ok(())
}

