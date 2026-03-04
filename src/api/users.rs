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
    Ok(Json(User::get_by_id(id)?).into_response())
}

pub async fn get_by_handle(
    Path(handle): Path<UserHandle>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    Ok(Json(User::get_by_handle(handle)?).into_response())
}

pub async fn get_all(headers: HeaderMap) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    Ok(Json(User::get_all()?).into_response())
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
    if !u.has_permission(Permission::ManuallyCreateUsers)? {
        return Ok((StatusCode::FORBIDDEN, CANT_MANUALLY_MAKE_USERS).into_response());
    }
    Ok(Json(User::create(form.handle)?).into_response())
}
pub async fn change_handle(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(form): Json<HandleForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let mut target = if u.id == id {
        u
    } else {
        if !u.has_permission(Permission::ChangeOthersHandles)? {
            return Ok((StatusCode::FORBIDDEN, CANT_CHANGE_OTHERS_HANDLE).into_response());
        }
        User::get_by_id(id)?
    };
    target.set_handle(form.handle)?;
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
    let mut target = if u.id == id {
        u
    } else {
        if !u.has_permission(Permission::ChangeOthersPasswords)? {
            return Ok((StatusCode::FORBIDDEN, CANT_CHANGE_OTHERS_PASSW).into_response());
        }
        User::get_by_id(id)?
    };
    target.set_password(Some(&form.password))?;
    Ok(PASSW_CHANGED_SUCCESS.into_response())
}
