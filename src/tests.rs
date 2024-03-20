use crate::config;
use config::Settings;
use std::path::Path;

#[cfg(test)]
#[test]
pub fn test_serialize() {

    let path = "tests/test_host_config";
    // first host
    let host1 = config::HostConfig::new(
        "user".to_string(),
        config::HostIdentifier::Ip(String::from("192.168.1.0/24")),
        22,
        Path::new("~/.ssh/testkey").to_path_buf(),
        Path::new("remote/path").to_path_buf(),
        Path::new("dest/path").to_path_buf(),
        24.0,
    );

    let settings: Settings = Settings::new(vec![host1]);
    settings.serialize_json("tests/test_host_config").unwrap();
}

#[test]
pub fn test_deserialize() {
    let path = "hosts";
    let settings = Settings::deserialize_json(path).unwrap();
}

