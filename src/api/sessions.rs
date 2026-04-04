use axum::{
    Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    api::CompositeError,
    database::{self, DatabaseError},
    logs::{LogAction, LogEntry},
    users::{
        User,
        auth::{UserAuthRequired, UserAuthenticate},
        permissions::Permission,
        sessions::{Session, SessionError},
    },
};

const CANT_REVOKE: &str = "You don't have permission to revoke this user's sessions.";

pub async fn get_by_id(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    let s = Session::get_by_id(&conn, id)?;

    match s.user_id == u.id
        || u.has_permission(&conn, Permission::ListOthersSessions)
            .is_ok_and(|v| v)
    {
        true => Ok(Json(s).into_response()),
        false => Err(SessionError::NoSessionWithId(id))?,
    }
}

pub async fn revoke_by_id(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let mut conn = database::conn()?;
    let tx = conn.transaction().map_err(DatabaseError::from)?;

    let mut s = Session::get_by_id(&tx, id)?;
    match s.user_id == u.id
        || u.has_permission(&tx, Permission::RevokeOthersSessions)
            .is_ok_and(|v| v)
    {
        true => {
            s.revoke(&tx, Some(&u))?;
            LogEntry::new(&tx, u, LogAction::ManuallyRevokeSession { id })?;
            tx.commit().map_err(DatabaseError::from)?;
            Ok(Json(s).into_response())
        }
        false => match u.has_permission(&tx, Permission::ListOthersSessions)? {
            true => Ok((StatusCode::FORBIDDEN, CANT_REVOKE).into_response()),
            false => Err(SessionError::NoSessionWithId(id))?,
        },
    }
}
