use std::fmt::Display;

use uuid::Uuid;

use crate::users::User;

pub struct LogEntry {
    pub id: Uuid,
    pub actor: User,
    pub data: LogAction,
}

#[derive(Debug)]
pub enum LogAction {
    Initialize,
    RegenInfradmin,
}

impl Display for LogAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogAction::Initialize => write!(f, "Initialized Mnemosyne."),
            LogAction::RegenInfradmin => write!(f, "Regenerated the Infradmin account."),
        }
    }
}
