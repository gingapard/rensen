use std::path::{Path, PathBuf};
use std::io::{self};
use std::fs;

use crate::logging::*;
use crate::snapshot::*;
use crate::utils::*;
use crate::traits::FileSerializable;

use crate::record::Record;

pub struct Compiler {
    pub snapshot: Snapshot,
}

impl Compiler {

    pub fn from<P>(record_path: P) -> Result<Self, Trap> 
    where 
        P: AsRef<Path>
    {
        let record = match Record::deserialize_json(record_path.as_ref()) {
            Ok(v) => v,
            Err(e) => {
                log_trap(Trap::FS, format!("Could not deserialize record: {}", e).as_str());
                return Err(Trap::FS);
            }
        };

        Ok(Compiler { snapshot: record.snapshot })
    }

    pub fn compile(&mut self, destination: &Path) -> Result<(), Trap> {
        // Directory at destination
        let _ = fs::create_dir_all(destination);

        for entry in &self.snapshot.entries {
            let path = &entry.1.path;
            let snapshot_path = &entry.1.snapshot_path;

            // if a demaked version of the snapshot does not already exist
            if !snapshot_path.exists() {
                let _ = demake_tar_gz(
                    format!("{}.tar.gz", entry.1.snapshot_path.as_path().to_str().unwrap()),
                    snapshot_path
                );  

                println!("damaked");
            }

            // The complete file destination 
            // (aka where it will collected with all other files in
            // the recored)
            let file_destination = replace_common_prefix(&path, &snapshot_path, &destination.to_path_buf());
            let _ = force_copy(&path, &file_destination);
            
            println!("copied");
        }

        Ok(())
    }

    /// Looping through entries and deleting all without the .tar.gz extension
    /// which where demaked (decompressed) in self.compile
    pub fn cleanup(&self) -> Result<(), Trap> {
        for entry in &self.snapshot.entries {
            let snapshot_path = strip_double_extension(&entry.1.snapshot_path);
            let _ = fs::remove_dir_all(snapshot_path);
        }
        
        Ok(())
    }
}

#[cfg(test)]
#[test]
pub fn test_compiler() {
    let path = PathBuf::from("/home/bam/backups/192.168.1.97/.records/2024-05-08-08-35-00Z.json");
    let mut compiler = Compiler::from(path).unwrap();

    let dest = PathBuf::from("/home/bam/snapshots");
    let _ = compiler.compile(dest.as_path());
}
