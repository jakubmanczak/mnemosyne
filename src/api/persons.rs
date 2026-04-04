use axum::{
    Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{self},
    error::CompositeError,
    logs::{LogAction, LogEntry},
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
    let conn = database::conn()?;
    Ok(Json(Person::get_all(&conn)?).into_response())
}
pub async fn get_by_id(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    Ok(Json(Person::get_by_id(&conn, id)?).into_response())
}
pub async fn pid_names(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    Ok(Json(Person::get_by_id(&conn, id)?.get_all_names(&conn)?).into_response())
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
    let mut conn = database::conn()?;
    let tx = conn.transaction()?;

    let p = Person::create(&tx, form.name, u.id)?;
    LogEntry::new(
        &tx,
        u,
        LogAction::CreatePerson {
            id: p.id,
            pname: p.primary_name.as_str().to_string(),
        },
    )?;
    tx.commit()?;
    Ok((StatusCode::CREATED, Json(p)).into_response())
}
pub async fn add_name(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(form): Json<PersonNameForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let mut conn = database::conn()?;
    let tx = conn.transaction()?;

    let p = Person::get_by_id(&tx, id)?;
    let n = p.add_name(&tx, form.name, u.id)?;
    LogEntry::new(
        &tx,
        u,
        LogAction::AddPersonName {
            pid: p.id,
            nid: n.id,
            pn: p.primary_name,
            nn: n.name.clone(),
        },
    )?;
    tx.commit()?;
    Ok((StatusCode::CREATED, Json(n)).into_response())
}

pub async fn n_by_id(Path(id): Path<Uuid>, headers: HeaderMap) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    Ok(Json(Name::get_by_id(&conn, id)?).into_response())
}
pub async fn n_setprimary(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let mut conn = database::conn()?;
    let tx = conn.transaction()?;

    if !u.has_permission(&tx, Permission::ChangePersonPrimaryName)? {
        return Ok((StatusCode::FORBIDDEN, CANT_SET_PRIMARYNAME).into_response());
    }

    let mut n = Name::get_by_id(&tx, id)?;
    let p = Person::get_by_id(&tx, n.person_id)?;
    n.set_primary(&tx)?;
    n.is_primary = true;
    LogEntry::new(
        &tx,
        u,
        LogAction::SetPersonPrimaryName {
            pid: p.id,
            nid: n.id,
            on: p.primary_name,
            nn: n.name.clone(),
        },
    )?;
    tx.commit()?;

    Ok(Json(n).into_response())
}
