use serde::{Serialize, Deserialize}; 
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use crate::traits::JsonFile;
use std::fmt::{Display, Formatter, Result};
use crate::snapshot::*;


/* listened to "Plastic Love" while coding this. */

/// A record storing the data for precompressed files.
#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub interval_n: u8,
    pub intervals: Vec<PathBuf>,
    pub snapshot: Snapshot,
}

impl Record {
    pub fn new() -> Self {
        Record {
            interval_n: 0,
            intervals: Vec::new(),
            snapshot: Snapshot::new(),
        }
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Record {{\n\tinterval_n: {},\n\tintervals: {:?},\n\tsnapshot: {}\n\t}}", self.interval_n, self.intervals, self.snapshot)
    }
}

impl JsonFile for Record {

    fn serialize_json(&self, file_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;
        let json_str = serde_json::to_string_pretty(&self)?;
        write!(file, "{}", json_str)?;
        Ok(())
    }

    fn deserialize_json(file_path: &Path) -> std::io::Result<Self> {
        let mut file = match File::open(file_path) {
            Ok(v) => v,
            Err(_) => {
                return Ok(Record::new());
            },
        };

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let record: Record = serde_json::from_str(&contents)?;
        Ok(record)
    }
}

#[cfg(test)]
#[test]
fn test_serialize_record() {
    /*
    let mut entries: HashMap<PathBuf, PathBufx> = HashMap::new();  
    entries.insert("/home/bam/backups/file1".into(), PathBufx::from(PathBuf::from("/home/cbroo/files/file1"), 123897));
    entries.insert("/home/bam/backups/file1".into(), PathBufx::from(PathBuf::from("/home/cbroo/files/file2"), 123897));
    entries.insert("/home/bam/backups/file1".into(), PathBufx::from(PathBuf::from("/home/cbroo/files/file3"), 123897));
    entries.insert("/home/bam/backups/file1".into(), PathBufx::from(PathBuf::from("/home/cbroo/files/file4"), 123897));
    */

    let mut record = Record::new();
    record.serialize_json(Path::new("record.json")).unwrap();
}

#[test]
fn test_deserialize_record() {
    let record: Record = Record::deserialize_json(Path::new("tests/record.json")).unwrap();
}
