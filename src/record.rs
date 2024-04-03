use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{self, prelude::*};
use crate::traits::FileSerializable;

/* listened to "Plastic Love" while coding this. */

#[derive(Serialize, Deserialize)]
pub struct Record {
    entries: Vec<(String, u64)>,
    len: usize,
}

impl FileSerializable for Record {

    fn serialize_json(&self, file_path: &str) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;
        let json_str = serde_json::to_string_pretty(&self)?;
        write!(file, "{}", json_str)?;
        Ok(())
    }

    fn deserialize_json(file_path: &str) -> std::io::Result<Self> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let record: Record = serde_json::from_str(&contents)?;
        Ok(record)
    }

    // yaml
    fn serialize_yaml(&self, file_path: &str) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;
        let yaml_str = serde_yaml::to_string(&self)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        let formatted_yaml = serde_yaml::to_string(&serde_yaml::from_str::<serde_yaml::Value>(&yaml_str)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))? 
        ).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        file.write_all(formatted_yaml.as_bytes())?;
        Ok(())
    }

    fn deserialize_yaml(file_path: &str) -> std::io::Result<Self> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let record: Record = serde_yaml::from_str(&contents)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        Ok(record)
    }
}
