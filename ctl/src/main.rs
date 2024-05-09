// std/other
use std::env;
use std::io::{self, Write, BufRead};
use console::{Style, Term};

// rensen-lib
use rensen_lib::logging::*;

// Action
mod action;
use action::*;

struct Ctl {}
impl Ctl {

    /// Starts the rensen-cli
    fn start(&mut self) -> Result<(), io::Error> {

        println!("Rensen - v0.1, GPL-3.0\n");

        loop {

            let input = self.get_input("<rensen> "); 

            let action = match self.parse_action_type(input?) {
                Some(action) => action,
                None => continue,
            };

            if action.action_type == ActionType::Exit {
                break;
            }

            // do_action


        }
        
        Ok(())
    }

    fn parse_action_type(&self, input: Vec<String>) -> Option<Action> {
        if input.is_empty() {
            return None;
        }

        let action_type = match input[0].to_lowercase().as_str() {
            "add"    => ActionType::Add,
            "remove" => ActionType::Remove,
            "show"   => ActionType::Show,
            "run"    => ActionType::Run,

            "help"   => ActionType::Help,

            "exit"   => ActionType::Exit,
            "quit"   => ActionType::Exit,
            "q"      => ActionType::Exit,
            _ => return None,
        };

        Some(Action { action_type, operands: input.iter().skip(1).cloned().collect() })
    }

    /// Prints promt and stdin
    fn get_input(&self, prompt: &str) -> Result<Vec<String>, io::Error> {
        print!("{}", prompt);
        io::stdout().flush()?; 

        let mut buffer = String::new();
        io::stdin().lock().read_line(&mut buffer)?; 

        Ok(buffer.split_whitespace().map(String::from).collect()) // Split input and collect into Vec<String> for parsing later
    }

}

fn main() {
    let mut ctl = Ctl {};
    let term = Term::stdout();
    term.clear_screen().unwrap();
    let _ = ctl.start();
}
