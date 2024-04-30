use std::path::{Path, PathBuf};

use crate::record::*;
use crate::snapshot::*;
use crate::utils::*;

pub struct Compiler {
    record: Record
}

impl From<Record> for Compiler {
    fn from(record: Record) -> Self {
        Compiler { record }
    }
}
