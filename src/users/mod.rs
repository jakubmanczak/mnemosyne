use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rusqlite::{OptionalExtension, ffi::SQLITE_CONSTRAINT_UNIQUE};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    ISE_MSG,
    database::{self, DatabaseError},
    users::{
        auth::UserPasswordHashing,
        handle::{UserHandle, UserHandleError},
    },
};

pub mod auth;
pub mod handle;
pub mod permissions;
pub mod sessions;
pub mod setup;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub handle: UserHandle,
}

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("UserHandleError: {0}")]
    UserHandleError(#[from] UserHandleError),
    #[error("No user found with ID {0}")]
    NoUserWithId(Uuid),
    #[error("No user found with handle {0}")]
    NoUserWithHandle(UserHandle),
    #[error("A user with this handle already exists")]
    HandleAlreadyExists,
    #[error("{0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Argon2 passhash error: {0}")]
    PassHashError(argon2::password_hash::Error),
}

impl User {
    pub fn get_by_id(id: Uuid) -> Result<User, UserError> {
        let res = database::conn()?
            .prepare("SELECT handle FROM users WHERE id = ?1")?
            .query_one((&id,), |r| {
                Ok(User {
                    id,
                    handle: r.get(0)?,
                })
            })
            .optional()?;
        match res {
            Some(u) => Ok(u),
            None => Err(UserError::NoUserWithId(id)),
        }
    }
    pub fn get_by_handle(handle: UserHandle) -> Result<User, UserError> {
        let res = database::conn()?
            .prepare("SELECT id, handle FROM users WHERE handle = ?1")?
            .query_one((&handle,), |r| {
                Ok(User {
                    id: r.get(0)?,
                    handle: r.get(1)?,
                })
            })
            .optional()?;
        match res {
            Some(u) => Ok(u),
            None => Err(UserError::NoUserWithHandle(handle)),
        }
    }
    pub fn set_handle(&mut self, new_handle: UserHandle) -> Result<(), UserError> {
        let conn = database::conn()?;
        conn.prepare("UPDATE users SET handle = ?1 WHERE id = ?2")?
            .execute((&new_handle, self.id))?;
        self.handle = new_handle;
        Ok(())
    }
}

// DANGEROUS: AUTH
impl User {
    pub fn set_password(&mut self, passw: Option<&str>) -> Result<(), UserError> {
        let conn = database::conn()?;
        match passw {
            None => {
                conn.prepare("UPDATE users SET password = NULL WHERE id = ?1")?
                    .execute((self.id,))?;
                Ok(())
            }
            Some(passw) => {
                let hashed = User::hash_password(passw)?;
                conn.prepare("UPDATE users SET password = ?1 WHERE id = ?2")?
                    .execute((hashed, self.id))?;
                Ok(())
            }
        }
    }
}

// RESERVED USERS IMPL
impl User {
    /// Constructs and pushes an infradmin to database
    ///
    /// An infradmin is the user account made for controlling
    /// Mnemosyne top-down. The infrastructure admin has permission
    /// to do everything and probably should not be used as a regular account
    /// due to the ramifications of compromise. But it could be used for that,
    /// and have its name changed.
    pub fn create_infradmin() -> Result<User, UserError> {
        let mut u = User {
            id: Uuid::max(),
            handle: UserHandle::new("Infradmin")?,
        };
        database::conn()?
            .prepare("INSERT INTO users(id, handle) VALUES (?1, ?2)")?
            .execute((&u.id, &u.handle))?;
        u.regenerate_infradmin_password()?;

        Ok(u)
    }

    /// Checks if the User is an infradmin
    ///
    /// An infradmin is the user account made for controlling
    /// Mnemosyne top-down. The infrastructure admin has permission
    /// to do everything and probably should not be used as a regular account
    /// due to the ramifications of compromise. But it could be used for that,
    /// and have its name changed.
    pub fn is_infradmin(&self) -> bool {
        self.id == Uuid::max()
    }

    /// Regenerates the infradmin password
    ///
    /// An infradmin is the user account made for controlling
    /// Mnemosyne top-down. The infrastructure admin has permission
    /// to do everything and probably should not be used as a regular account
    /// due to the ramifications of compromise. But it could be used for that,
    /// and have its name changed.
    pub fn regenerate_infradmin_password(&mut self) -> Result<(), UserError> {
        let passw = auth::generate_token(auth::TokenSize::Char16);
        self.set_password(Some(&passw))?;
        log::info!("[USERS] The infradmin account password has been (re)generated.");
        log::info!("[USERS] Handle: {}", self.handle.as_str());
        log::info!("[USERS] Password: {}", passw);
        log::info!("[USERS] The infradmin is urged to change this password to a secure one.");
        Ok(())
    }

    /// Constructs and pushes a systemuser to database
    ///
    /// A systemuser is used for internal blame representation
    /// for actions performed by Mnemosyne internally.
    /// It shall not be available for log-in.
    /// It should not have its name changed, and should be protected from that.
    pub fn create_systemuser() -> Result<User, UserError> {
        let u = User {
            id: Uuid::nil(),
            handle: UserHandle::new("Mnemosyne")?,
        };
        database::conn()?
            .prepare("INSERT INTO users(id, handle) VALUES (?1, ?2)")?
            .execute((&u.id, &u.handle))?;

        Ok(u)
    }

    /// Checks if the User is a systemuser
    ///
    /// A systemuser is used for internal blame representation
    /// for actions performed by Mnemosyne internally.
    /// It shall not be available for log-in.
    /// It should not have its name changed, and should be protected from that.
    pub fn is_systemuser(&self) -> bool {
        self.id == Uuid::nil()
    }
}

impl From<rusqlite::Error> for UserError {
    fn from(error: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(err, Some(msg)) = &error
            && err.extended_code == SQLITE_CONSTRAINT_UNIQUE
            && msg.contains("handle")
        {
            return UserError::HandleAlreadyExists;
        }
        UserError::DatabaseError(DatabaseError::from(error))
    }
}
impl From<argon2::password_hash::Error> for UserError {
    fn from(err: argon2::password_hash::Error) -> Self {
        UserError::PassHashError(err)
    }
}
impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        match self {
            Self::DatabaseError(e) => e.into_response(),
            Self::PassHashError(e) => {
                log::error!("[PASSHASH] A passwordhash error occured: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, ISE_MSG.to_string()).into_response()
            }
            Self::UserHandleError(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            Self::NoUserWithId(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            Self::NoUserWithHandle(_) => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
            Self::HandleAlreadyExists => (StatusCode::CONFLICT, self.to_string()).into_response(),
        }
    }
}
