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
    persons::{Name, Person},
    users::{
        User,
        auth::{UserAuthRequired, UserAuthenticate},
        permissions::Permission,
    },
};

pub const CANT_SET_PRIMARYNAME: &str = "You don't have permission to swap primary names.";

pub async fn get_all(headers: HeaderMap) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    Ok(Json(Person::get_all()?).into_response())
}
pub async fn get_by_id(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    Ok(Json(Person::get_by_id(id)?).into_response())
}
pub async fn pid_names(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    Ok(Json(Person::get_by_id(id)?.get_all_names()?).into_response())
}

#[derive(Deserialize)]
pub struct PersonNameForm {
    name: String,
}

pub async fn create(
    headers: HeaderMap,
    Json(form): Json<PersonNameForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let p = Person::create(form.name, u.id)?;
    Ok((StatusCode::CREATED, Json(p)).into_response())
}
pub async fn add_name(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(form): Json<PersonNameForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let p = Person::get_by_id(id)?;
    let n = p.add_name(form.name, u.id)?;

    Ok((StatusCode::CREATED, Json(n)).into_response())
}

pub async fn n_by_id(Path(id): Path<Uuid>, headers: HeaderMap) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    Ok(Json(Name::get_by_id(id)?).into_response())
}
pub async fn n_setprimary(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    if !u.has_permission(Permission::ChangePersonPrimaryName)? {
        return Ok((StatusCode::FORBIDDEN, CANT_SET_PRIMARYNAME).into_response());
    }

    let mut n = Name::get_by_id(id)?;
    n.set_primary()?;
    n.is_primary = true;
    Ok(Json(n).into_response())
}
