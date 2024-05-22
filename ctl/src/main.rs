// std/other
use std::env; use std::io::{self, Write, BufRead};
use console::{Term, Style};
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
    pub global_config: GlobalConfig,
    pub term: Term
}

impl Ctl {

    /// Starts the rensen-cli
    fn start(&mut self) -> Result<(), io::Error> {

        let style = Style::new();

        println!("Rensen Alpha v1.0\nLicense: GPL-3.0\n");
        println!("{}", style.clone().bold().apply_to("                                                             
             d8888b. d88888b d8b   db .d8888. d88888b d8b   db 
             88  `8D 88'     888o  88 88'  YP 88'     888o  88 
             88oobY' 88ooooo 88V8o 88 `8bo.   88ooooo 88V8o 88 
             88`8b   88~~~~~ 88 V8o88   `Y8b. 88~~~~~ 88 V8o88 
             88 `88. 88.     88  V888 db   8D 88.     88  V888 
             88   YD Y88888P VP   V8P `8888Y' Y88888P VP   V8P 
        "));

        println!("This software comes with {}, to the extent permitted by applicable law.\n", style.clone().bold().underlined().apply_to("ABSOLUTELY NO WARRANTY"));

        loop {

            let input = match get_input("<rensen> ") {
                Ok(input) => {
                    if input.len() < 2 { continue };
                    input
                },
                Err(err) => return Err(err)
            };

            // convert to Vec<String> by splitting at whitespace
            let input_vec: Vec<String> = input.split_whitespace().map(String::from).collect();

            // Checking the first index, what type of action it predicts.
            let action = match self.parse_action_type(&input_vec) {
                Some(action) => {
                    action
                }
                None => {
                    println!("`{}` is not a recognized action!", input_vec[0]);
                    continue;
                }
            };

            if action.action_type == ActionType::Exit {
                println!("bye");
                break;
            }

            // execute the action of commnad given
            let _ = match action.execute() {
                Ok(_) => (),
                Err(e) => println!("{:?}", e)
            };
        }
        
        Ok(())
    }

    fn parse_action_type(&self, input: &Vec<String>) -> Option<Action> {
        if input.is_empty() {
            return None;
        }

        let action_type = match input[0].to_lowercase().as_str() {
            "a" | "add"           => ActionType::AddHost,
            "d" | "del"           => ActionType::DeleteHost,
            "l" | "list"          => ActionType::List,
            "r" | "run"           => ActionType::RunBackup,
            "c" | "compile"       => ActionType::Compile,
            "h" | "?" | "help"    => ActionType::Help,
            "q" | "quit" | "exit" => ActionType::Exit,
            _ => return None,
        };

        let global_config = self.global_config.clone();
        Some(Action { global_config, action_type, operands: input.iter().skip(1).cloned().collect() })
    }

    pub fn clear_screen(&self) {
        self.term.clear_screen().unwrap();
    }
}

fn main() -> std::io::Result<()> {
    let global_config_path = PathBuf::from("/etc/rensen/rensen_config.yml");
    let mut ctl = Ctl { 
        global_config: match GlobalConfig::deserialize_yaml(&global_config_path) {
            Ok(v) => v,
            Err(err) => {
                println!("{}", err);
                return Ok(());
            }
        },

        term: Term::stdout()

    };

    ctl.clear_screen();
    let _ = ctl.start();

    Ok(())
}
