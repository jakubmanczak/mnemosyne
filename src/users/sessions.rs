use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    ISE_MSG, database,
    users::{User, auth},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expiry: DateTime<Utc>,
    #[serde(flatten)]
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, strum::EnumIs)]
#[serde(tag = "revoked")]
pub enum SessionStatus {
    #[serde(rename = "false")]
    Active,
    #[serde(rename = "true")]
    Revoked {
        revoked_at: DateTime<Utc>,
        revoked_by: Uuid,
    },
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum SessionError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("No session found with id: {0}")]
    NoSessionWithId(Uuid),
    #[error("No session found with token: {0}")]
    NoSessionWithToken(String),
}
impl From<rusqlite::Error> for SessionError {
    fn from(error: rusqlite::Error) -> Self {
        SessionError::DatabaseError(error.to_string())
    }
}
impl IntoResponse for SessionError {
    fn into_response(self) -> Response {
        match self {
            Self::DatabaseError(e) => {
                eprintln!("[ERROR] Database error occured: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, ISE_MSG.into())
            }
            Self::NoSessionWithId(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::NoSessionWithToken(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        }
        .into_response()
    }
}

impl Session {
    pub fn get_by_id(id: Uuid) -> Result<Session, SessionError> {
        let res = database::conn()?
            .prepare("SELECT user_id, expiry, revoked, revoked_at, revoked_by FROM sessions WHERE id = ?1")?
            .query_one((&id,), |r| Ok(Session {
                id: id,
                user_id: r.get(0)?,
                expiry: r.get(1)?,
                status: match r.get::<_, bool>(2)? {
                    false => SessionStatus::Active,
                    true => {
                        SessionStatus::Revoked { revoked_at: r.get(3)?, revoked_by: r.get(4)? }
                    }
                }
            })).optional()?;

        match res {
            Some(s) => Ok(s),
            None => Err(SessionError::NoSessionWithId(id)),
        }
    }
    pub fn get_by_token(token: &str) -> Result<Session, SessionError> {
        let hashed = Sha256::digest(token.as_bytes()).to_vec();
        let res = database::conn()?
            .prepare("SELECT id, user_id, expiry, revoked, revoked_at, revoked_by FROM sessions WHERE token = ?1")?
            .query_one((hashed,), |r| Ok(Session {
                id: r.get(0)?,
                user_id: r.get(1)?,
                expiry: r.get(2)?,
                status: match r.get::<_, bool>(3)? {
                    false => SessionStatus::Active,
                    true => {
                        SessionStatus::Revoked { revoked_at: r.get(4)?, revoked_by: r.get(5)? }
                    }
                }
            })).optional()?;

        match res {
            Some(s) => Ok(s),
            None => Err(SessionError::NoSessionWithToken(token.to_string())),
        }
    }
    pub fn new_for_user(user: &User) -> Result<(Session, String), SessionError> {
        let id = Uuid::now_v7();
        let token = auth::generate_token(auth::TokenSize::Char64);
        let hashed = Sha256::digest(token.as_bytes()).to_vec();
        let expiry = Utc::now() + Session::DEFAULT_PROLONGATION;

        database::conn()?
            .prepare("INSERT INTO sessions VALUES (?1, ?2, ?3, ?4)")?
            .execute((&id, &hashed, user.id, expiry))?;
        let s = Session {
            id,
            user_id: user.id,
            expiry,
            status: SessionStatus::Active,
        };
        Ok((s, token))
    }

    const DEFAULT_PROLONGATION: Duration = Duration::days(14);
    const PROLONGATION_THRESHOLD: Duration = Duration::hours(2);
    pub fn prolong(&mut self) -> Result<(), SessionError> {
        if self.expiry - Session::DEFAULT_PROLONGATION + Session::PROLONGATION_THRESHOLD
            > Utc::now()
        {
            return Ok(());
        }

        let expiry = Utc::now() + Session::DEFAULT_PROLONGATION;
        database::conn()?
            .prepare("UPDATE sessions SET expiry = ?1 WHERE id = ?2")?
            .execute((&expiry, &self.id))?;
        self.expiry = expiry;
        Ok(())
    }

    pub fn revoke(&mut self, actor: Option<&User>) -> Result<(), SessionError> {
        let now = Utc::now();
        let id = actor.map(|u| u.id).unwrap_or(Uuid::nil());
        database::conn()?
            .prepare(
                "UPDATE sessions SET revoked = ?1, revoked_at = ?2, revoked_by = ?3 WHERE id = ?4",
            )?
            .execute((&true, &now, &id, &self.id))?;
        self.status = SessionStatus::Revoked {
            revoked_at: now,
            revoked_by: id,
        };
        Ok(())
    }

    pub fn issued(&self) -> DateTime<Utc> {
        // unwrapping here since we use UUIDv7
        // and since we assume we're not in 10k CE
        let timestamp = self.id.get_timestamp().unwrap().to_unix();
        DateTime::from_timestamp_secs(timestamp.0 as i64).unwrap()
    }
    pub fn is_expired_or_revoked(&self) -> bool {
        self.is_expired() || self.status.is_revoked()
    }
    pub fn is_expired(&self) -> bool {
        self.expiry <= Utc::now()
    }
}
