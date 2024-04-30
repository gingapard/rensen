use std::path::{Path, PathBuf};
use std::io::{self};

use crate::record::*;
use crate::logging::*;
use crate::snapshot::*;
use crate::utils::*;

pub struct Compiler {
    pub record: Record
}

impl From<Record> for Compiler {
    fn from(record: Record) -> Self {
        Compiler { record }
    }
}

impl Compiler {
    
    pub fn compile_from_record(&self) -> Result<(), Trap> {

        Ok(())
    }


}
