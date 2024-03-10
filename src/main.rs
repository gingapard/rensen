pub mod conf;
pub mod logging;
pub mod utils;
use conf::serde;
use std::fs::File;
use std::path::Path;

use std::{env, net, io::Result, error};
use env_logger;

fn main() -> Result<()> {
    env_logger::init();

    if let Err(err) = utils::zip_compress_dir("mydir", "mydir.tar.gz") {
        eprintln!("Error: {}", err);
    }
    else {
        println!("zipped dir");
    }
    Ok(())
}
