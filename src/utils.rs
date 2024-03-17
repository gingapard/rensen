use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::io::prelude::*;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;
use sha3::{Digest, Sha3_256};

use crate::logging;
use logging::{log_error, ErrorType};

pub fn zip_compress_dir(path: &str, output_file_path: &str) -> io::Result<()> {
    // temp tar file
    let tar_file_path = "temp.tar";
    let tar_file = File::create(tar_file_path)?;

    // Create a tarball
    let mut tar_builder = Builder::new(tar_file);
    add_dir_contents_to_tar(path, &mut tar_builder, path)?;
    tar_builder.finish()?;

    // gzip compress
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

pub fn hash_file(file_path: &Path) -> Result<String, ErrorType> {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => {
            log_error(ErrorType::FS, format!("Could not open {:?}: {}", file_path, err).as_str());
            return Err(ErrorType::FS);
        }
    };


    let mut buffer = [0; 1024];
    match file.read_exact(&mut buffer) {
        Ok(_) => (),
        Err(err) => {
            log_error(ErrorType::FS, format!("Could not read to buffer: {}", err).as_str());
            return Err(ErrorType::FS);
        }
    }

    Ok("".to_string())
}


