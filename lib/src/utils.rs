use std::fs::{self, File};
use std::io::{self, SeekFrom, BufReader, BufWriter, Read};
use std::path::{Path, PathBuf}; use std::io::prelude::*;
use flate2::{write::GzEncoder, read::GzDecoder};
use flate2::Compression;
use tar::{Builder, Archive};
use sha3::{Digest, Sha3_256};
use std::os::unix::fs::PermissionsExt;
use std::time::{SystemTime, Duration};
use ssh2::FileStat;
use chrono::offset;
use termion::cursor;

use crate::logging;
use logging::Trap;

use crate::traits::ConvertFromPath;

pub fn get_datetime() -> String {
    return offset::Local::now()
        .format("%Y-%m-%d-%H-%M-%SZ")
        .to_string()
}

pub fn get_file_sz<P>(path: P) -> u64
where 
    P: AsRef<Path> 
{
    match fs::metadata(path) {
        Ok(metadata) => {
            return metadata.len()
        }
        _ => ()
    }

    0
}

pub fn clear_current_line() {
    print!("\x1B[1A\x1B[K\x1B[1A\x1B[K\x1B[1B");
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

/// Recurses a dir and returns number of files that it contains
pub fn count_files<P>(path: P) -> Result<usize, Trap> 
where 
    P: AsRef<Path> + std::fmt::Debug + Copy
{
    let mut sum = 0;
    let dir = match fs::read_dir(path) {
        Ok(dir) => dir,
        Err(err) => return Err(Trap::FS(format!("Could not read dir {:?}: {}", path, err)))
    };

    for entry in dir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => return Err(Trap::FS(format!("Could not read file: {}", err)))
        }.path();

        if entry.is_dir() {
            sum += count_files(&entry)?;
        }
        else if entry.is_file() {
            sum += 1;
        }
    }

    Ok(sum)
}

#[test]
fn test_count_files() {
    let path = "/etc/rensen";
    let files = count_files(path).unwrap();

    assert_eq!(files, 9);
}

/// Archive directory with Tarball (tar::Builder) and
/// compress with Gz (flate2::write::GzeEncoder, flate2::Compression).
///
/// source: path for directory to compress
/// destination: path to compressed and archived file
pub fn make_tar_gz<SRC, DST>(source: SRC, destination: DST) -> io::Result<()>
where 
    SRC: AsRef<Path>,
    DST: AsRef<Path>
{
    let source = source.as_ref();
    let destination = destination.as_ref();

    let mut files_added = 0;
    let file_count = count_files(source).unwrap();
    println!("Archiving: ({}/{})", 0, file_count);

    // Temp tar file
    let tar_file_path = "temp.tar";
    let tar_file = File::create(tar_file_path)?;

    // Create a tarball
    let mut tar_builder = Builder::new(tar_file);
    add_dir_contents_to_tar(source, &mut tar_builder, source, &mut files_added, &file_count)?;
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
    files_added: &mut i32,
    file_count: &usize
) -> io::Result<()> {

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(root).unwrap().to_string_lossy().into_owned();

        if path.is_dir() {
            tar_builder.append_dir(name, &path)?;
            add_dir_contents_to_tar(root, tar_builder, &path, files_added, file_count)?;
        } else {
            *files_added += 1;
            clear_current_line();
            println!("Archiving: ({}/{})", files_added, file_count );
            tar_builder.append_path_with_name(&path, name)?;
        }
    }

    Ok(())
}

// Decompresses and dearchives .tar.gz 
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

/// Pops filestem and adds it back without the last
/// dot-extension.
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
pub fn strip_double_extension(path: &Path) -> PathBuf {
    let mut path = path.to_path_buf();
    strip_extension(&mut path);
    strip_extension(&mut path);
    return path;
}

#[test]
fn test_strip_tar_gz_extension() {
    let original = PathBuf::from("path/to/certain/path.tar.gz");
    let new = strip_double_extension(&original);
    assert!(original == new,"{:?} - {:?}", original, new);
}

/// Replaces the prefix part of 
/// path1 and path2 with somehting else
pub fn replace_common_prefix(one: &PathBuf, two: &PathBuf, replacement: &PathBuf) -> PathBuf {
    let common_prefix = one.components()
        .zip(two.components())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a)
        .collect::<Vec<_>>()
    ;

    let mut new_path = PathBuf::from(replacement);
    for component in one.iter().skip(common_prefix.len()) {
        new_path.push(component);
    }
    
    return new_path;
}

/// Wrapper for std::fs::copy which forces the write by
/// creating missing directories
pub fn force_copy(source: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    println!("{:?}", source);
    println!("{:?}", destination);

    // Create destination directory if it doesn't exist
    if let Some(parent_dir) = destination.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }
    }

    let mut source_file = File::open(source)?;
    let mut destination_file = File::create(destination)?;
    io::copy(&mut source_file, &mut destination_file)?;

    Ok(())
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
        Trap::FS(format!("Could not open {:?}: {}", path, err))
    })?;

    let mut sha3_256 = Sha3_256::new();
    let mut buffer = [0; 1024];

    file.seek(SeekFrom::Start(pos)).map_err(|err| {
        Trap::FS(format!("Could not seek in {:?}: {}", path, err))
    })?;

    match file.read(&mut buffer) {
        Ok(bytes_read) => {
            sha3_256.update(&buffer[..bytes_read]);
        }
        Err(err) => {
            return Err(Trap::FS(format!("Could not read from {:?}: {}", path, err)));
        }
    }

    Ok(format!("{:x}", sha3_256.finalize()))
}
