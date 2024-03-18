use std::fs::{self, File};
use std::io::{self, Seek, SeekFrom,  BufReader, BufWriter};
use std::path::Path;
use std::io::prelude::*;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;
use sha3::{Digest, Sha3_256};

use crate::logging;
use logging::{log_error, ErrorType};

/// Archive directory with Tarball (tar::Builder) and
/// compress with Gz (flate2::write::GzeEncoder, flate2::Compression).
pub fn archive_compress_dir(path: &str, output_file_path: &str) -> io::Result<()> {
    // Temp tar file
    let tar_file_path = "temp.tar";
    let tar_file = File::create(tar_file_path)?;

    // Create a tarball
    let mut tar_builder = Builder::new(tar_file);
    add_dir_contents_to_tar(path, &mut tar_builder, path)?;
    tar_builder.finish()?;

    // Gzip compress
    let tar_file = File::open(tar_file_path)?;
    let gz_file = File::create(output_file_path)?;
    let gz_encoder = GzEncoder::new(BufWriter::new(gz_file), Compression::default());
    io::copy(&mut BufReader::new(tar_file), &mut BufWriter::new(gz_encoder))?;

    // Cleanup: remove temp tar file
    fs::remove_file(tar_file_path)?;

    Ok(())
}

fn add_dir_contents_to_tar(
    root: &str,
    tar_builder: &mut Builder<File>,
    dir: &str,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(root).unwrap().to_string_lossy().into_owned();

        if path.is_dir() {
            tar_builder.append_dir(name, &path)?;
            add_dir_contents_to_tar(root, tar_builder, &path.to_string_lossy())?;
        } else {
            tar_builder.append_path_with_name(&path, name)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn test_hash() {
    let path = Path::new("src/hosts");
    match hash_file(path, 0) {
        Ok(hash) => println!("Hash: {}", hash),
        Err(err) => panic!("Error: {:?}", err),
    }
}

/// Read the next 1024 bytes from the 'pos'-th byte.
pub fn hash_file(path: &Path, pos: u64) -> Result<String, ErrorType> {
    let mut file = File::open(path).map_err(|err| {
        log_error(ErrorType::FS, format!("Could not open {:?}: {}", path, err).as_str());
        ErrorType::FS
    })?;

    let mut sha3_256 = Sha3_256::new();
    let mut buffer = [0; 1024];

    file.seek(SeekFrom::Start(pos)).map_err(|err| {
        log_error(ErrorType::FS, format!("Could not seek in {:?}: {}", path, err).as_str());
        ErrorType::FS
    })?;

    match file.read(&mut buffer) {
        Ok(bytes_read) => {
            sha3_256.update(&buffer[..bytes_read]);
        }
        Err(err) => {
            log_error(ErrorType::FS, format!("Could not read from {:?}: {}", path, err).as_str());
            return Err(ErrorType::FS);
        }
    }

    Ok(format!("{:x}", sha3_256.finalize()))
}
