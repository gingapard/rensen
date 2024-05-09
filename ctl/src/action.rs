
#[derive(PartialEq)]
pub enum ActionType {
    Add,    // (1 arg)
    Remove, // (1 arg)
    Run,    // (2 arg)
    Show,   // (2 arg)
    
    Help,   // (0 arg)
    Exit,   // (0 arg)
}

pub struct Action {
    pub action_type: ActionType,
    pub operands: Vec<String>,
}

impl Action {
    pub fn do_action(&self) {
    }
}
