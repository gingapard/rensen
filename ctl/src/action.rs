use rensen_lib::logging::Trap;
use rensen_lib::config::*;
use rensen_lib::traits::{Rsync, FileSerializable};

use crate::utils::*;

use std::path::Path;

#[derive(PartialEq)]
pub enum ActionType {
    AddHost,    // (1 arg)
    RemoveHost, // (1 arg)
    ModifyHost,
    RunTask,    // (2 arg)
    Show,       // (2 arg)
    
    Help,       // (0 arg)
    Exit,       // (0 arg)
}

pub struct Action {
    pub action_type: ActionType,
    pub operands: Vec<String>,
}

impl Action {
    pub fn do_action(&self) -> Result<(), Trap> {

        match self.action_type {
            ActionType::AddHost => {
                self.add_host()?;
            },
            // TODO: add rest of actions.

            _ => (),
        }

        Ok(())
    }

    fn add_host(&self) -> Result<(), Trap> {
        // global settings for rensen
        let mut settings = Settings::deserialize_yaml(Path::new(""));
        
        let user          = get_input("user: ")
            .map_err(|err| Trap::Missing);
        let identifier    = get_input("identifier (addr): ");
        let port          = get_input("port (leave empty for 22): ");
        let key_path      = get_input("ssh-key: ");
        let source        = get_input("source: ");
        let destination   = get_input("destination: ");
        let frequency_hts = get_input("backup frquency (hrs): ");

        // TODO: Serialize 

        Ok(())
    }
}




