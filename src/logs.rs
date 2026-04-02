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
    CreateUser { id: Uuid, handle: String },
    CreateTag { id: Uuid, name: String },
    CreatePerson { id: Uuid, pname: String },
    ChangeUserHandle { id: Uuid, old: String, new: String },
}

impl Display for LogAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogAction::Initialize => write!(f, "Initialized Mnemosyne."),
            LogAction::RegenInfradmin => write!(f, "Regenerated the Infradmin account."),
            LogAction::CreateUser { id, handle } => {
                write!(f, "Created user @{handle} (uid: {id})")
            }
            LogAction::CreateTag { id, name } => {
                write!(f, "Created tag #{name} (id: {id})")
            }
            LogAction::CreatePerson { id, pname } => {
                write!(f, "Created person ~{pname} (id: {id})")
            }
            LogAction::ChangeUserHandle { id, old, new } => {
                write!(f, "Changed user handle @{old} -> @{new} (uid: {id})")
            }
        }
    }
}
