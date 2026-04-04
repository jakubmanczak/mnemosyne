use axum::{
    Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::CompositeError,
    database::{self, DatabaseError},
    logs::{LogAction, LogEntry},
    users::{
        User,
        auth::{UserAuthRequired, UserAuthenticate},
        handle::UserHandle,
        permissions::Permission,
    },
};

const CANT_CHANGE_OTHERS_HANDLE: &str = "You don't have permission to change this user's handle.";
const CANT_CHANGE_OTHERS_PASSW: &str = "You don't have permission to change this user's password.";
const CANT_MANUALLY_MAKE_USERS: &str = "You don't have permission to manually create new users.";
const HANDLE_CHANGED_SUCCESS: &str = "Handle changed successfully.";
const PASSW_CHANGED_SUCCESS: &str = "Password changed successfully.";

pub async fn get_me(headers: HeaderMap) -> Result<Response, CompositeError> {
    Ok(Json(User::authenticate(&headers)?.required()?).into_response())
}

pub async fn get_by_id(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    Ok(Json(User::get_by_id(&conn, id)?).into_response())
}

pub async fn get_by_handle(
    Path(handle): Path<UserHandle>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    Ok(Json(User::get_by_handle(&conn, handle)?).into_response())
}

pub async fn get_all(headers: HeaderMap) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    Ok(Json(User::get_all(&conn)?).into_response())
}

#[derive(Deserialize)]
pub struct HandleForm {
    handle: UserHandle,
}
pub async fn create(
    headers: HeaderMap,
    Json(form): Json<HandleForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let mut conn = database::conn()?;
    let tx = conn.transaction().map_err(DatabaseError::from)?;

    if !u.has_permission(&tx, Permission::ManuallyCreateUsers)? {
        return Ok((StatusCode::FORBIDDEN, CANT_MANUALLY_MAKE_USERS).into_response());
    }

    let nu = User::create(&tx, form.handle)?;
    LogEntry::new(
        &tx,
        u,
        LogAction::CreateUser {
            id: nu.id,
            handle: nu.handle.as_str().to_string(),
        },
    )?;
    tx.commit().map_err(DatabaseError::from)?;

    Ok(Json(nu).into_response())
}
pub async fn change_handle(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(form): Json<HandleForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let mut conn = database::conn()?;
    let tx = conn.transaction().map_err(DatabaseError::from)?;

    let mut target = if u.id == id {
        u.clone()
    } else {
        if !u.has_permission(&tx, Permission::ChangeOthersHandles)? {
            return Ok((StatusCode::FORBIDDEN, CANT_CHANGE_OTHERS_HANDLE).into_response());
        }
        User::get_by_id(&tx, id)?
    };

    let old_handle = target.handle.as_str().to_string();
    target.set_handle(&tx, form.handle)?;
    LogEntry::new(
        &tx,
        u,
        LogAction::ChangeUserHandle {
            id: target.id,
            old: old_handle,
            new: target.handle.as_str().to_string(),
        },
    )?;
    tx.commit().map_err(DatabaseError::from)?;

    Ok(HANDLE_CHANGED_SUCCESS.into_response())
}

#[derive(Deserialize)]
pub struct ChangePasswordForm {
    password: String,
}
pub async fn change_password(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(form): Json<ChangePasswordForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let mut conn = database::conn()?;
    let tx = conn.transaction().map_err(DatabaseError::from)?;

    let mut target = if u.id == id {
        u.clone()
    } else {
        if !u.has_permission(&tx, Permission::ChangeOthersPasswords)? {
            return Ok((StatusCode::FORBIDDEN, CANT_CHANGE_OTHERS_PASSW).into_response());
        }
        User::get_by_id(&tx, id)?
    };

    target.set_password(&tx, Some(&form.password))?;
    LogEntry::new(
        &tx,
        u,
        LogAction::ManuallyChangeUsersPassword { id: target.id },
    )?;
    tx.commit().map_err(DatabaseError::from)?;

    Ok(PASSW_CHANGED_SUCCESS.into_response())
}
