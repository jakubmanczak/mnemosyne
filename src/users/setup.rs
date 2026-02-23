use rusqlite::OptionalExtension;
use uuid::Uuid;

use crate::{
    database,
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
        User::create_systemuser()?;
    }

    if conn
        .prepare("SELECT handle FROM users WHERE id = ?1")?
        .query_one((&Uuid::max(),), |_| Ok(()))
        .optional()?
        .is_none()
    {
        User::create_infradmin()?;
    }

    Ok(())
}
