use crate::logging;
use logging::Trap;
use std::path::Path;

pub trait YamlFile: Sized { 
    /// Wrapper for serde::yaml
    fn serialize_yaml(&self, file_path: &Path) -> std::io::Result<()>;
    /// Wrapper for serde::yaml
    fn deserialize_yaml(file_path: &Path) -> std::io::Result<Self>;
}

pub trait JsonFile: Sized {
    /// Wrapper for serde::json
    fn serialize_json(&self, file_path: &Path) -> std::io::Result<()>;
    /// Wrapper for serde::json
    fn deserialize_json(file_path: &Path) -> std::io::Result<Self>;
}

pub trait Rsync {
    fn backup(&mut self) -> Result<(), Trap>;
    fn auth(&mut self) -> Result<(), Trap>;
    fn connect(&mut self) -> Result<(), Trap>;
    fn copy_remote_directory(&self, remote_path: &Path, dest_path: &Path) -> Result<(), Trap>;
    fn copy_remote_file(&self, remote_path: &Path, dest_path: &Path) -> Result<(), Trap>;
}

pub trait ConvertFromPath {
    fn convert_from_path(path: &Path) -> Self;
}

