use rusqlite::OptionalExtension;
use uuid::Uuid;

use crate::{
    database,
    logs::{LogAction, LogEntry},
    users::{User, UserError},
};

pub fn initialise_reserved_users_if_needed() -> Result<(), UserError> {
    let mut conn = database::conn()?;
    let tx = conn.transaction()?;

    if tx
        .prepare("SELECT handle FROM users WHERE id = ?1")?
        .query_one((&Uuid::nil(),), |_| Ok(()))
        .optional()?
        .is_none()
    {
        let u = User::create_systemuser(&tx)?;
        LogEntry::new(&tx, u, LogAction::Initialize)?;
    }

    if tx
        .prepare("SELECT handle FROM users WHERE id = ?1")?
        .query_one((&Uuid::max(),), |_| Ok(()))
        .optional()?
        .is_none()
    {
        User::create_infradmin(&tx)?;
        LogEntry::new(
            &tx,
            User::get_by_id(&tx, Uuid::nil())?,
            LogAction::RegenInfradmin,
        )?;
    }

    tx.commit()?;
    Ok(())
}
