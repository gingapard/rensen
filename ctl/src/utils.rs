use std::io::{self, Write, BufRead};

pub fn get_input(prompt: &str) -> Result<Vec<String>, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?; 

    let mut buffer = String::new();
    io::stdin().lock().read_line(&mut buffer)?; 

    Ok(buffer.split_whitespace().map(String::from).collect()) // Split input and collect into Vec<String> for parsing later
}
