use std::path::{Path, PathBuf};
use std::fs;

use crate::logging::*;
use crate::snapshot::*; use crate::utils::*; use crate::traits::JsonFile;
use crate::utils::make_tar_gz;

use crate::record::Record;

pub struct Compiler {
    pub source_snapshot_path: PathBuf,
    pub source_snapshot: Snapshot,
}

impl Compiler {

    pub fn from(record_path: &PathBuf) -> Result<Self, Trap> {
        let record = match Record::deserialize_json(record_path.as_ref()) {
            Ok(record) => record,
            Err(err) => {
                return Err(Trap::FS(format!("Could not deserialize record: {}", err)));
            }
        };

        let mut record_path = record_path.clone();
        strip_extension(&mut record_path);
        Ok(Compiler { source_snapshot_path: record_path.to_path_buf(), source_snapshot: record.snapshot })
    } 

    /// Compiles from self.snapshot to destination
    /// note: destination has to be
    /// full path (including file + extension)
    pub fn compile(&mut self, destination: &Path) -> Result<(), Trap> {
        // Directory at destination

        print!("Compiling ...");

        let full_destination = destination.join(self.source_snapshot_path.file_name().unwrap());
        let _ = fs::create_dir_all(&full_destination);

        for entry in &self.source_snapshot.entries {
            let file_path = &entry.1.file_path;
            let snapshot_path = &entry.1.snapshot_path;

            // if a demaked version of the snapshot does not already exist
            if !snapshot_path.exists() {
                let _ = demake_tar_gz(
                    format!("{}.tar.gz", entry.1.snapshot_path.as_path().to_str().unwrap()),
                    snapshot_path
                );  
            }

            // The complete file destination 
            // (aka where it will collected with all other files in
            // the recored)
            let file_destination = replace_common_prefix(&file_path, &snapshot_path, &full_destination.to_path_buf());
            let _ = force_copy(&file_path, &file_destination);

        }

        // Because `full_snapshot_path` is the `source` in this matter.
        make_tar_gz(&full_destination, format!("{}.tar.gz", full_destination.to_str().unwrap()))
            .map_err(|err| Trap::FS(format!("Could not archive and compress snapshot: {}", err)))?;

        println!("Done");
        Ok(())
    }

    /// Looping through entries and deleting all without the .tar.gz extension
    /// which where demaked (decompressed) in self.compile
    pub fn cleanup(&self) -> Result<(), Trap> {
        for entry in &self.source_snapshot.entries {
            let snapshot_path = strip_double_extension(&entry.1.snapshot_path);
            let _ = fs::remove_dir_all(snapshot_path);
        }
        
        Ok(())
    }
}

// TODO: Test compiler
#[test]
fn test_compiler() {
    let path = Path::new("/home/bam/backups/192.168.1.97/.records/2024-05-15-08-10-30Z.json");
    let mut compiler = Compiler::from(&path.to_path_buf()).unwrap();

    let snapshot_path = Path::new("/home/bam/snapshots");
    compiler.compile(snapshot_path).unwrap();

}
