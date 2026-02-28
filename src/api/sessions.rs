use axum::{
    Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    api::CompositeError,
    users::{
        User,
        auth::{UserAuthRequired, UserAuthenticate},
        permissions::Permission,
        sessions::{Session, SessionError},
    },
};

const CANT_REVOKE: &str = "You don't have permission to change this user's password.";

pub async fn get_by_id(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let s = Session::get_by_id(id)?;

    match s.user_id == u.id
        || u.has_permission(Permission::ListOthersSessions)
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
    let mut s = Session::get_by_id(id)?;

    match s.user_id == u.id
        || u.has_permission(Permission::RevokeOthersSessions)
            .is_ok_and(|v| v)
    {
        true => {
            s.revoke(Some(&u))?;
            Ok(Json(s).into_response())
        }
        false => match u.has_permission(Permission::ListOthersSessions)? {
            true => Ok((StatusCode::FORBIDDEN, CANT_REVOKE).into_response()),
            false => Err(SessionError::NoSessionWithId(id))?,
        },
    }
}
