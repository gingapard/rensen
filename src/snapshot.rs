use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Wrapper for PathBuf holding its mtime as u64
#[derive(Debug, Serialize, Deserialize)]
pub struct PathBufx {
    pub path: PathBuf,
    pub mtime: u64,
}

/// Entries containing the mtime of files.
/// Using the source path as key, we can get data.
#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub entries: HashMap<PathBuf, PathBufx>,
    pub deleted_entries: HashSet<PathBuf>
}

impl Snapshot {
    pub fn new() -> Self {
        Snapshot {
            entries: HashMap::new(),
            deleted_entries: HashSet::new(),
        }
    }

    pub fn add_entry(&mut self, path: PathBuf, local_path: PathBuf, mtime: u64) {
        self.entries.insert(path, PathBufx {path: local_path, mtime});
    }

    pub fn mark_as_deleted(&mut self, path: PathBuf) {
        self.deleted_entries.insert(path);
    }

    pub fn is_deleted(&self, path: &PathBuf) -> bool {
        self.deleted_entries.contains(path)
    }

    pub fn mtime(&self, path: &PathBuf) -> Option<&u64> {
        Some(&self.entries.get(path).unwrap().mtime)
    }
}
