use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::fmt::{Display, Result, Formatter};

/// Wrapper for PathBuf holding its mtime as u64
#[derive(Debug, Serialize, Deserialize)]
pub struct PathBufx {
    pub file_path: PathBuf, 
    pub snapshot_path: PathBuf, // root path (no extension)
    pub mtime: u64,
}

impl PathBufx {
    pub fn new() -> Self {
        PathBufx {
            file_path: PathBuf::new(),
            snapshot_path: PathBuf::new(),
            mtime: u64::MIN,
        }
    }

    pub fn from(file_path: PathBuf, snapshot_path: PathBuf, mtime: u64) -> Self {
        PathBufx {
            file_path,
            snapshot_path,
            mtime,
        }
    }
}

/// Containg two pairing (equal) paths
/// the local path (destination) and it's equivelent remote path (source)
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct PathPair {
    pub source: PathBuf,
    pub destination: PathBuf,
}

impl PathPair {
    pub fn from(source: PathBuf, destination: PathBuf) -> Self {
        PathPair {
            source,
            destination
        }
    }
}

/// Entries containing the mtime of files.
/// Using the source path as key, we can get data.
#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub entries: HashMap<PathBuf, PathBufx>,
    pub deleted_entries: HashSet<PathPair>
}

impl Display for Snapshot {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Snapshot: {{\n\tentries: {:?},\n\tdeleted_entries: {:?}\n\t\n}}", self.entries, self.deleted_entries)
    }
}

impl Snapshot {
    pub fn new() -> Self {
        Snapshot {
            entries: HashMap::new(),
            deleted_entries: HashSet::new(),
        }
    }

    pub fn add_entry(&mut self, pathpair: PathPair, snapshot_path: PathBuf, mtime: u64) {
        self.entries.insert(pathpair.source, PathBufx::from(pathpair.destination, snapshot_path, mtime));
    }

    pub fn mark_as_deleted(&mut self, pair: PathPair) {
        self.entries.remove(&pair.source);
        self.deleted_entries.insert(pair);
    }

    pub fn is_deleted(&self, pair: &PathPair) -> bool {
        self.deleted_entries.contains(pair)
    }

    pub fn undelete(&mut self, pair: &PathPair) {
        self.deleted_entries.remove(pair);
    }

    /// returns the mtime entry matching key
    pub fn mtime(&self, key: &PathBuf) -> Option<&u64> {
        if let Some(entry) = &self.entries.get(key) {
            return Some(&entry.mtime)
        }

        None
    }

    pub fn path(&self, key: &PathBuf) -> Option<&PathBuf> {
        if let Some(entry) = &self.entries.get(key) {
            return Some(&entry.file_path);
        }

        None
    }
}

