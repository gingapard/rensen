// std/other
use std::env; use std::io::{self, Write, BufRead};
use console::{Style, Term};
use std::path::{Path, PathBuf};

// rensen-lib
use rensen_lib::logging::*;
use rensen_lib::config::GlobalConfig;
use rensen_lib::traits::YamlFile;

// Action
pub mod action;
use action::*;

pub mod utils;
use utils::*;

#[derive(Debug, Clone)]
struct Ctl {
    pub global_config: GlobalConfig
}

impl Ctl {

    /// Starts the rensen-cli
    fn start(&mut self) -> Result<(), io::Error> {

        println!("Rensen v0.1\nLicense: GPL-3.0\n");

        loop {

            // Getting immi input and convert to Vec<String> by splitting at whitespace
            let input_vec: Vec<String> = get_input("<rensen> ")?.split_whitespace().map(String::from).collect();

            let action = match self.parse_action_type(input_vec) {
                Some(action) => action,
                None => continue,
            };

            if action.action_type == ActionType::Exit {
                println!("bye");
                break;
            }

            // execute
            let _ = match action.execute() {
                Ok(_) => (),
                Err(e) => println!("{:?}", e)
            };

        }
        
        Ok(())
    }

    fn parse_action_type(&self, input: Vec<String>) -> Option<Action> {
        if input.is_empty() {
            return None;
        }

        let action_type = match input[0].to_lowercase().as_str() {
            "add"     => ActionType::AddHost,
            "remove"  => ActionType::RemoveHost,

            "list"    => ActionType::List,
            "show"    => ActionType::List,

            "run"     => ActionType::RunBackup,
            "compile" => ActionType::Compile,

            "help"    => ActionType::Help,
            "?"       => ActionType::Help,

            "exit"    => ActionType::Exit,
            "quit"    => ActionType::Exit,
            "q"       => ActionType::Exit,
            _ => return None,
        };

        let global_config = self.global_config.clone();
        Some(Action { global_config, action_type, operands: input.iter().skip(1).cloned().collect() })
    }
}

fn main() {
    let global_config_path = PathBuf::from("/etc/rensen/rensen_config.yml");
    let mut ctl = Ctl { global_config: match GlobalConfig::deserialize_yaml(&global_config_path) {
        Ok(v) => v,
        Err(err) => {
            println!("{}", err);
            return;
        }
    }};

    let term = Term::stdout();
    term.clear_screen().unwrap();
    let _ = ctl.start();
}
