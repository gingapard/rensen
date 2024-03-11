use std::io::{Read, Write};
use std::{path::Path, fs::File};
use std::net::TcpStream;
use ssh2::Session;

use crate::conf::serde;
use crate::logging::{ErrorType, InfoType, log_error, log_info};

fn connect_ssh(identifier: &str, port: &u16) -> Result<Session, ErrorType> {
    // connect
    let tcp = TcpStream::connect(format!("{}:{}", identifier, port)).map_err(|err| {
        log_error(ErrorType::Connect,
            format!("({}) Could not connect to host: {:?}", identifier, err).as_str());
        ErrorType::Connect
    })?;

    // open session
    let mut sess = Session::new().map_err(|err| {
        log_error(ErrorType::Session,
            format!("({}) Could not create SSH session: {:?}", identifier, err).as_str());
        ErrorType::Session
    })?;

    // handshake
    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(|err| {
        log_error(ErrorType::Handshake,
            format!("({}) Could not perform SSH handshake: {:?}", identifier, err).as_str());
        ErrorType::Handshake
    })?;

    Ok(sess)
}

pub fn backup_rsync(host: &mut serde::Host) -> Result<(), ErrorType> {
    // ext ip or hostname
    let identifier = match &host.identifier {
        serde::HostIdentifier::Ip(ip) => ip,
        serde::HostIdentifier::Hostname(hostname) => hostname,
    };

    // ext port
    let port = host.port.unwrap_or(22);
    let mut sess = connect_ssh(&identifier, &port)?;

    // read key
    let private_key_path = Path::new(host.key_path.as_ref().map_or("/", |s| s.to_str().unwrap_or("/")));
    let mut file = File::open(&private_key_path).map_err(|err| {
        log_error(ErrorType::KeyLoad,
            format!("({}) Could not read private key file: {:?}", err, host.identifier).as_str());
        ErrorType::KeyLoad
    })?;

    let mut private_key = String::new();
    file.read_to_string(&mut private_key).map_err(|err| {
        log_error(ErrorType::KeyLoad,
            format!("({}) Could not load private key file: {:?}", err, identifier).as_str());
        ErrorType::KeyLoad
    })?;

    // auth
    sess.userauth_pubkey_file(
        &host.user,
        None,
        Path::new("/"),
        Some(&private_key),
    ).map_err(|err| {
        log_error(ErrorType::Auth,
            format!("({}) Could not authenticate with private key: {:?}", err, identifier).as_str());
        ErrorType::Auth
    })?;

    // copy files
    if let Ok(mut ch) = sess.scp_recv(&host.remote_path) {
        // TODO handle copying of files
    } else {
        log_error(ErrorType::Channel,
            format!("Could not open channel for {}", identifier).as_str());
        return Err(ErrorType::Channel);
    }

    Ok(())
}







