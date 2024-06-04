use std::io::{self, Write, BufRead};
use std::fmt;
use std::path::PathBuf;

pub fn get_input(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?; 

    let mut buffer = String::new();
    io::stdin().lock().read_line(&mut buffer)?; 

    Ok(buffer)
}

#[derive(PartialEq, Debug)]
pub enum ByteUnit {
    B,
    Kib,
    Mib,
    Gib,
    Tib
}

impl ByteUnit {
    fn from_u32(value: u32) -> ByteUnit {
        match value {
            0 => ByteUnit::B,
            1 => ByteUnit::Kib,
            2 => ByteUnit::Mib,
            3 => ByteUnit::Gib,
            4 => ByteUnit::Tib,
            _ => ByteUnit::B,
        }
    }
}

impl fmt::Display for ByteUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ByteUnit::B => write!(f, "bytes"),
            ByteUnit::Kib => write!(f, "Kib"),
            ByteUnit::Mib => write!(f, "MiB"),
            ByteUnit::Gib => write!(f, "Gib"),
            ByteUnit::Tib => write!(f, "Tib"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct MemoryUsage {
    pub amount: f32,
    pub unit: ByteUnit
}

pub fn format_bytes(bytes: u64) -> MemoryUsage { 
    let mut current_amount = bytes;
    let mut unit = 0;
    let mut result: MemoryUsage = MemoryUsage { amount: 0.0, unit: ByteUnit::B };

    while current_amount > 1024 {
        current_amount /= 1024;
        unit += 1;

        if current_amount < 1024 {
            break;
        }
    }
    
    result.amount = current_amount as f32;
    result.unit = ByteUnit::from_u32(unit);
    return result;
}

#[cfg(test)]
#[test]
fn test_format_bytes() {
    let bytes: u64 = 900012000;
    assert_eq!(format_bytes(bytes), MemoryUsage { amount: 0.0, unit: ByteUnit::B });
}

pub fn sort_dates(dates: &mut Vec<PathBuf>) -> () {


    ()
}
