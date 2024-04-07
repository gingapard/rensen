use serde::{Serialize, Deserialize};
use std::fs::File;
use std::path::Path;
use std::io::{self, prelude::*};
use std::collections::HashMap;
use crate::traits::FileSerializable;


#[cfg(test)]
#[test]
fn test_serialize_record() {
    let mut entries: HashMap<String, u64> = HashMap::new();  
    entries.insert("/home/bam/backups/file1".to_string(), 90);
    entries.insert("/home/bam/backups/file2".to_string(), 1238947);
    entries.insert("/home/bam/backups/file3".to_string(), 239847298);
    entries.insert("/home/bam/backups/file4".to_string(), 2398129837);
    entries.insert("/home/bam/backups/file5".to_string(), 9812837123);

    let record = Record::new(entries);
    record.serialize_json(Path::new("record.json")).unwrap();
}

#[test]
fn test_deserialize_record() {
    let record: Record = Record::deserialize_json(Path::new("tests/record.json")).unwrap();
    println!("{:?}", record.entries);
}

/* listened to "Plastic Love" while coding this. */

/// A record storing the data for precompressed files.
#[derive(Serialize, Deserialize)]
pub struct Record {
    entries: HashMap<String, u64>,
}

impl Record {
    fn new(entries: HashMap<String, u64>) -> Self {
        Self { entries }
    }
}

impl FileSerializable for Record {

    fn serialize_json(&self, file_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;
        let json_str = serde_json::to_string_pretty(&self)?;
        write!(file, "{}", json_str)?;
        Ok(())
    }

    fn deserialize_json(file_path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let record: Record = serde_json::from_str(&contents)?;
        Ok(record)
    }

    // yaml
    fn serialize_yaml(&self, file_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;
        let yaml_str = serde_yaml::to_string(&self)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        let formatted_yaml = serde_yaml::to_string(&serde_yaml::from_str::<serde_yaml::Value>(&yaml_str)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))? 
        ).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        file.write_all(formatted_yaml.as_bytes())?;
        Ok(())
    }

    fn deserialize_yaml(file_path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let record: Record = serde_yaml::from_str(&contents)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        Ok(record)
    }
}
