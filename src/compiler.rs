use std::path::{Path, PathBuf};
use std::io::{self};

use crate::logging::*;
use crate::snapshot::*;
use crate::utils::*;
use crate::traits::FileSerializable;

use crate::record::Record;

/// Compiler for compiling snapshots into one,
/// using the .inner.join record of a specific snapshot.
pub struct Compiler {
    pub snapshot: Snapshot,
    pub inner_path: PathBuf
}

impl Compiler {

    pub fn from(snapshot_path: PathBuf) -> Result<Self, Trap> {
        let stripped_path = strip_tar_gz_extension(&snapshot_path); // removing the .tar.gz

        let _ = demake_tar_gz(&snapshot_path, &stripped_path).map_err(|err| {
            log_trap(Trap::FS, format!("Could not demake {:?}: {}", snapshot_path, err).as_str());
            Trap::FS
        });

        let inner_record_path = &stripped_path.join(".inner.json");
        let record = match Record::deserialize_json(inner_record_path) {
            Ok(v) => v,
            Err(err) => {
                log_trap(Trap::FS, format!("Could not deserialize inner record: {}", err).as_str());
                return Err(Trap::FS);
            }
        };
            

        Ok(Compiler { snapshot: record.snapshot, inner_path: inner_record_path.to_path_buf() })
    }

    /// Compiling a snapshot according to self.snapshot
    pub fn compile_snapshot(&self) -> Result<(), Trap> {
        
        for entry in self.snapshot.entries.iter() {
        }

        Ok(())
    }


}

#[cfg(test)]
#[test]
pub fn test_compiler() {
    let compiler = Compiler::from("/home/bam/backups/192.168.1.76/2024-05-02-06-42-09Z.tar.gz".into());
}
