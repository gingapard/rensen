use std::collections::{HashMap, HashSet};
use std::path::{PathBuf, Path};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub entries: HashMap<PathBuf, u64>,
    pub deleted_entries: HashSet<PathBuf>
}

impl Snapshot {
    pub fn new() -> Self {
        Snapshot {
            entries: HashMap::new(),
            deleted_entries: HashSet::new(),
        }
    }

    pub fn add_entry(&mut self, path: PathBuf, last_modified: u64) {
        self.entries.insert(path, last_modified);
    }

    pub fn mark_as_deleted(&mut self, path: PathBuf) {
        self.deleted_entries.insert(path);
    }

    pub fn is_deleted(&self, path: &PathBuf) -> bool {
        self.deleted_entries.contains(path)
    }

    pub fn mtime(&self, path: &PathBuf) -> Option<&u64> {
        self.entries.get(path)
    }
}
