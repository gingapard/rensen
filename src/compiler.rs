use std::path::{Path, PathBuf};
use std::io::{self};

use crate::record::*;
use crate::logging::*;
use crate::snapshot::*;
use crate::utils::*;

pub struct Compiler {
    pub snapshot_path: PathBuf,
    pub snapshot: Snapshot
}

impl Compiler {

    pub fn from(snapshot_path: PathBuf) -> Result<Self, Trap> {
        let stripped_path = strip_extension(&snapshot_path); // removing the .tar.gz

        let _ = demake_tar_gz(&snapshot_path, stripped_path).map_err(|err| {
            log_trap(Trap::FS, format!("Could not demake {:?}: {}", snapshot_path, err).as_str());
            Trap::FS
        });

            

        Ok(Compiler { snapshot_path, snapshot: Snapshot::new() })
    }

    pub fn compile_from_record(&self, destination: &Path) -> Result<(), Trap> {

        Ok(())
    }


}
