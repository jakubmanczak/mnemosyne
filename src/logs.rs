use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;
use uuid::Uuid;

use crate::{
    database::{self, DatabaseError},
    users::User,
};

#[derive(Debug)]
pub struct LogEntry {
    pub id: Uuid,
    pub actor: User,
    pub data: LogAction,
}

impl LogEntry {
    pub fn new(actor: User, data: LogAction) -> Result<LogEntry, DatabaseError> {
        let log = LogEntry {
            id: Uuid::now_v7(),
            actor,
            data,
        };
        let conn = database::conn()?;
        let actiontype: &'static str = (&log.data).into();
        let payload = serde_json::to_string(&log.data).unwrap();
        conn.prepare(
            "INSERT INTO logs(id, actor, target, actiontype, payload) VALUES (?1,?2,?3,?4,?5)",
        )?
        .execute((
            &log.id,
            &log.actor.id,
            log.data.get_target_id(),
            actiontype,
            payload,
        ))?;
        Ok(log)
    }
    pub fn get_all() -> Result<Vec<LogEntry>, DatabaseError> {
        Ok(database::conn()?
            .prepare("SELECT id, actor, target, actiontype, payload FROM logs ORDER BY id DESC")?
            .query_map((), |r| {
                let payload: String = r.get(4)?;
                Ok(LogEntry {
                    id: r.get(0)?,
                    actor: User::get_by_id(r.get(1)?).unwrap(),
                    data: serde_json::from_str(&payload).unwrap(),
                })
            })?
            .collect::<Result<Vec<LogEntry>, _>>()?)
    }
}

// #[derive(Debug, thiserror::Error)]
// pub enum LogError {}

#[derive(Debug, IntoStaticStr, Serialize, Deserialize)]
pub enum LogAction {
    Initialize,
    RegenInfradmin,
    CreateUser { id: Uuid, handle: String },
    CreateTag { id: Uuid, name: String },
    CreatePerson { id: Uuid, pname: String },
    ChangeUserHandle { id: Uuid, old: String, new: String },
}
impl LogAction {
    pub fn get_target_id(&self) -> Option<Uuid> {
        match self {
            Self::Initialize | Self::RegenInfradmin => None,
            Self::CreateUser { id, .. }
            | Self::CreateTag { id, .. }
            | Self::CreatePerson { id, .. }
            | Self::ChangeUserHandle { id, .. } => Some(*id),
        }
    }
    pub fn get_humanreadable_payload(&self) -> String {
        match self {
            LogAction::Initialize => format!("Initialized Mnemosyne."),
            LogAction::RegenInfradmin => format!("Regenerated the Infradmin account."),
            LogAction::CreateUser { id, handle } => {
                format!("Created user @{handle} (uid: {id})")
            }
            LogAction::CreateTag { id, name } => {
                format!("Created tag #{name} (id: {id})")
            }
            LogAction::CreatePerson { id, pname } => {
                format!("Created person ~{pname} (id: {id})")
            }
            LogAction::ChangeUserHandle { id, old, new } => {
                format!("Changed user handle @{old} -> @{new} (uid: {id})")
            }
        }
    }
}
