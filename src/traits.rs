use crate::logging;
use logging::{ErrorType, log_error};
use std::path::Path;

pub trait FileSerializable: Sized { 
    /// Wrapper for serde::json
    fn serialize_json(&self, file_path: &Path) -> std::io::Result<()>;
    /// Wrapper for serde::json
    fn deserialize_json(file_path: &Path) -> std::io::Result<Self>;
    /// Wrapper for serde::yaml
    fn serialize_yaml(&self, file_path: &Path) -> std::io::Result<()>;
    /// Wrapper for serde::yaml
    fn deserialize_yaml(file_path: &Path) -> std::io::Result<Self>;
}

pub trait BackupMethod {
    fn full_backup(&mut self) -> Result<(), ErrorType>;
    fn incremental_backup(&mut self) -> Result<(), ErrorType>;
    fn auth(&mut self) -> Result<(), ErrorType>;
    fn connect(&mut self) -> Result<(), ErrorType>;
    fn copy_remote_directory(&mut self, remote_path: &Path, dest_path: &Path) -> Result<(), ErrorType>;
    fn copy_remote_file(&mut self, remote_path: &Path, dest_path: &Path) -> Result<(), ErrorType>;
}
