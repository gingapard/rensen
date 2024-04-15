use crate::config;
use crate::traits::FileSerializable;
use config::Settings;
use std::path::Path;

#[cfg(test)]
#[test]
pub fn test_serialize_json() {

    let path = Path::new("../tests/test.json");
    // first host
    let host1 = config::HostConfig::new(
        "user".to_string(),
        config::HostIdentifier::Ip(String::from("192.168.1.0/24")),
        22,
        Path::new("~/.ssh/testkey").to_path_buf(),
        Path::new("remote/path").to_path_buf(),
        Path::new("dest/path").to_path_buf(),
        24.0,
        true,
    );

    let settings: Settings = Settings::new(vec![host1]);
    settings.serialize_json(path).unwrap();
}

#[test]
pub fn test_deserialize_json() {
    let path = Path::new("tests/test.json");
    let settings = Settings::deserialize_json(path).unwrap();
}

#[test]
pub fn test_serialize_yaml() {
    let path = Path::new("tests/test.yaml");
    let host1 = config::HostConfig::new(
        "user".to_string(),
        config::HostIdentifier::Ip(String::from("192.168.1.0/24")),
        22,
        Path::new("~/.ssh/testkey").to_path_buf(),
        Path::new("remote/path").to_path_buf(),
        Path::new("dest/path").to_path_buf(),
        24.0,
        false,
    );
    
    let host2 = config::HostConfig::new(
        "user2".to_string(),
        config::HostIdentifier::Ip(String::from("192.168.1.0/24")),
        22,
        Path::new("~/.ssh/testkey").to_path_buf(),
        Path::new("remote/path").to_path_buf(),
        Path::new("dest/path").to_path_buf(),
        24.0,
        false,
    );

    let settings: Settings = Settings::new(vec![host1, host2]);
    settings.serialize_yaml(path).unwrap();
}

#[test]
pub fn test_deserialize_yaml() {
    let path = Path::new("tests/test.yaml");
    let settings = Settings::deserialize_yaml(path).unwrap();
}

