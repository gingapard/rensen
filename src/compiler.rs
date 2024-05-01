use std::path::{Path, PathBuf};
use std::io::{self};

use crate::record::*;
use crate::logging::*;
use crate::snapshot::*;
use crate::utils::*;

pub struct Compiler {
    pub snapshot_path: PathBuf,
    pub record: Record 
}

impl Compiler {

    pub fn compile_from_record(&self, destination: &Path) -> Result<(), Trap> {

        Ok(())
    }


}
