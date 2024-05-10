use std::io::{self, Write, BufRead};

pub fn get_input(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?; 

    let mut buffer = String::new();
    io::stdin().lock().read_line(&mut buffer)?; 

    Ok(buffer)
}
