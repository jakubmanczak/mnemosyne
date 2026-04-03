use rusqlite::OptionalExtension;
use uuid::Uuid;

use crate::{
    database,
    logs::{LogAction, LogEntry},
    users::{User, UserError},
};

pub fn initialise_reserved_users_if_needed() -> Result<(), UserError> {
    let conn = database::conn()?;

    if conn
        .prepare("SELECT handle FROM users WHERE id = ?1")?
        .query_one((&Uuid::nil(),), |_| Ok(()))
        .optional()?
        .is_none()
    {
        let u = User::create_systemuser()?;
        LogEntry::new(u, LogAction::Initialize)?;
    }

    if conn
        .prepare("SELECT handle FROM users WHERE id = ?1")?
        .query_one((&Uuid::max(),), |_| Ok(()))
        .optional()?
        .is_none()
    {
        User::create_infradmin()?;
        LogEntry::new(User::get_by_id(Uuid::nil())?, LogAction::RegenInfradmin)?;
    }

    Ok(())
}
