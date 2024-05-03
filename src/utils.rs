use std::fs::{self, File};
use std::io::{self, SeekFrom, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use flate2::{write::GzEncoder, read::GzDecoder};
use flate2::Compression;
use tar::{Builder, Archive};
use sha3::{Digest, Sha3_256};
use std::os::unix::fs::{PermissionsExt, MetadataExt, FileExt};
use std::time::{SystemTime, Duration};
use ssh2::FileStat;
use chrono::Utc;

use crate::logging;
use logging::{log_trap, Trap};

use crate::traits::ConvertFromPath;

pub fn get_datetime() -> String {
  return Utc::now().format("%Y-%m-%d-%H-%M-%SZ").to_string()
}

/// Sets the metadata for $file according to $stat
pub fn set_metadata(file: &mut File, stat: FileStat) -> Result<(), Trap> {

    // len/size
    let _ = file.set_len(stat.size.unwrap_or(0));

    // Permissions
    if let Some(raw_perms) = stat.perm {
        let _ = file.set_permissions(PermissionsExt::from_mode(raw_perms));
    }

    // File Times
    let default_value: u64 = 0;

    let file_times = fs::FileTimes::new();
    file_times.set_accessed(SystemTime::UNIX_EPOCH + Duration::from_secs(stat.atime.unwrap_or(default_value)));
    file_times.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(stat.mtime.unwrap_or(default_value)));
    let _ = file.set_times(file_times);
    let _ = file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(stat.mtime.unwrap_or(default_value)));

    Ok(())
}

/// Archive directory with Tarball (tar::Builder) and
/// compress with Gz (flate2::write::GzeEncoder, flate2::Compression).
pub fn make_tar_gz<SRC, DST>(source: SRC, destination: DST) -> io::Result<()>
where 
    SRC: AsRef<Path>,
    DST: AsRef<Path>
{
    let source = source.as_ref();
    let destination = destination.as_ref();

    // Temp tar file
    let tar_file_path = "temp.tar";
    let tar_file = File::create(tar_file_path)?;

    // Create a tarball
    let mut tar_builder = Builder::new(tar_file);
    add_dir_contents_to_tar(source, &mut tar_builder, source)?;
    tar_builder.finish()?;

    // Gzip compress
    let tar_file = File::open(tar_file_path)?;
    let gz_file = File::create(destination)?;
    let gz_encoder = GzEncoder::new(BufWriter::new(gz_file), Compression::default());
    io::copy(&mut BufReader::new(tar_file), &mut BufWriter::new(gz_encoder))?;

    // Cleanup: remove temp tar file, remove uncompressed file
    let _ = fs::remove_dir_all(source);
    let _ = fs::remove_file(tar_file_path);

    Ok(())
}

/// Recurses dir and adds it to the root tar_builder.
fn add_dir_contents_to_tar(
    root: &Path,
    tar_builder: &mut Builder<File>,
    dir: &Path,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(root).unwrap().to_string_lossy().into_owned();

        if path.is_dir() {
            tar_builder.append_dir(name, &path)?;
            add_dir_contents_to_tar(root, tar_builder, &path)?;
        } else {
            tar_builder.append_path_with_name(&path, name)?;
        }
    }

    Ok(())
}

pub fn demake_tar_gz<SRC, DST>(source: SRC, destination: DST) -> io::Result<()>
where
    SRC: AsRef<Path>,
    DST: AsRef<Path>,
{
    let destination = destination.as_ref();

    let _ = fs::create_dir_all(destination);

    let gz_file = fs::File::open(source)?;
    let gz_decoder = GzDecoder::new(BufReader::new(gz_file));

    let mut archive = Archive::new(gz_decoder);
    archive.unpack(destination)?;

    Ok(())
}

impl ConvertFromPath for PathBuf {
    fn convert_from_path(path: &Path) -> Self {
        path.to_path_buf()
    }
}

impl ConvertFromPath for String {
    fn convert_from_path(path: &Path) -> Self {
        path.to_string_lossy().to_string()
    }
}

pub fn strip_extension(path: &mut PathBuf) {
    if let Some(stem) = path.clone().file_stem() {
        path.pop();
        path.push(stem);
    }
}

/// If the path has a file extension, it will remove the file extension 
/// and return the Some(S)
///
/// # Example:
///
/// (path/to/this/file.tar.gz -> path/to/this/file)
///
pub fn strip_tar_gz_extension(path: &Path) -> PathBuf {
    let mut path = path.to_path_buf();
    strip_extension(&mut path);
    strip_extension(&mut path);

    return path;
}

#[test]
fn test_strip_tar_gz_extension() {
    let original = PathBuf::from("path/to/certain/path.tar.gz");
    let new = strip_tar_gz_extension(&original);
    assert!(original == new,"{:?} - {:?}", original, new);
}

#[test]
fn test_hash() {
    let path = Path::new("src/hosts");
    match hash_file(path, 0) {
        Ok(hash) => println!("Hash: {}", hash),
        Err(err) => panic!("Error: {:?}", err),
    }
}

/// Read the next 1024 bytes from the 'pos'-th byte.
pub fn hash_file(path: &Path, pos: u64) -> Result<String, Trap> {
    let mut file = File::open(path).map_err(|err| {
        log_trap(Trap::FS, format!("Could not open {:?}: {}", path, err).as_str());
        Trap::FS
    })?;

    let mut sha3_256 = Sha3_256::new();
    let mut buffer = [0; 1024];

    file.seek(SeekFrom::Start(pos)).map_err(|err| {
        log_trap(Trap::FS, format!("Could not seek in {:?}: {}", path, err).as_str());
        Trap::FS
    })?;

    match file.read(&mut buffer) {
        Ok(bytes_read) => {
            sha3_256.update(&buffer[..bytes_read]);
        }
        Err(err) => {
            log_trap(Trap::FS, format!("Could not read from {:?}: {}", path, err).as_str());
            return Err(Trap::FS);
        }
    }

    Ok(format!("{:x}", sha3_256.finalize()))
}
