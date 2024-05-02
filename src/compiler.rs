use std::path::{Path, PathBuf};
use std::io::{self};

use crate::logging::*;
use crate::snapshot::*;
use crate::utils::*;
use crate::traits::FileSerializable;

use crate::record::Record;


pub struct Compiler {
    pub snapshot: Snapshot
}

impl Compiler {

    pub fn from(snapshot_path: PathBuf) -> Result<Self, Trap> {
        let stripped_path = strip_extension(&snapshot_path); // removing the .tar.gz

        let _ = demake_tar_gz(&snapshot_path, &stripped_path).map_err(|err| {
            log_trap(Trap::FS, format!("Could not demake {:?}: {}", snapshot_path, err).as_str());
            Trap::FS
        });

        let record = match Record::deserialize_json(&stripped_path.join(".inner.json")) {
            Ok(v) => v,
            Err(err) => {
                log_trap(Trap::FS, format!("Could not deserialize inner record: {}", err).as_str());
                return Err(Trap::FS);
            }
        };
            

        Ok(Compiler { snapshot: record.snapshot })
    }

    pub fn compile_from_record(&self, destination: &Path) -> Result<(), Trap> {

        Ok(())
    }


}
