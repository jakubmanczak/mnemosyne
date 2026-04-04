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
    tags::{Tag, TagName},
    users::{
        User,
        auth::{UserAuthRequired, UserAuthenticate},
        permissions::Permission,
    },
};

const CANT_MAKE_TAGS: &str = "You don't have permission to create new tags.";
const CANT_DEL_TAGS: &str = "You don't have permission to delete tags.";
const CANT_RENAME_TAGS: &str = "You don't have permission to rename tags.";
const TAG_DELETED: &str = "Tag deleted successfully.";

pub async fn get_all(headers: HeaderMap) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    Ok(Json(Tag::get_all(&conn)?).into_response())
}

pub async fn get_by_id(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    Ok(Json(Tag::get_by_id(&conn, id)?).into_response())
}

pub async fn get_by_name(
    Path(name): Path<TagName>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    Ok(Json(Tag::get_by_name(&conn, name)?).into_response())
}

#[derive(Deserialize)]
pub struct TagNameForm {
    name: TagName,
}
pub async fn create(
    headers: HeaderMap,
    Json(form): Json<TagNameForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let mut conn = database::conn()?;
    let tx = conn.transaction()?;

    if !u.has_permission(&tx, Permission::CreateTags)? {
        return Ok((StatusCode::FORBIDDEN, CANT_MAKE_TAGS).into_response());
    }

    let t = Tag::create(&tx, form.name)?;
    LogEntry::new(
        &tx,
        u,
        LogAction::CreateTag {
            id: t.id,
            name: t.name.as_str().to_string(),
        },
    )?;
    tx.commit()?;
    Ok(Json(t).into_response())
}

pub async fn rename(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(form): Json<TagNameForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let mut conn = database::conn()?;
    let tx = conn.transaction()?;

    if !u.has_permission(&tx, Permission::RenameTags)? {
        return Ok((StatusCode::FORBIDDEN, CANT_RENAME_TAGS).into_response());
    }
    let mut tag = Tag::get_by_id(&tx, id)?;
    let on = tag.name.as_str().to_string();
    tag.rename(&tx, form.name)?;
    LogEntry::new(
        &tx,
        u,
        LogAction::RenameTag {
            id,
            on,
            nn: tag.name.as_str().to_string(),
        },
    )?;
    tx.commit()?;

    Ok(Json(tag).into_response())
}

pub async fn delete(Path(id): Path<Uuid>, headers: HeaderMap) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let mut conn = database::conn()?;
    let tx = conn.transaction()?;

    if !u.has_permission(&tx, Permission::DeleteTags)? {
        return Ok((StatusCode::FORBIDDEN, CANT_DEL_TAGS).into_response());
    }
    let t = Tag::get_by_id(&tx, id)?;
    let name = t.name.as_str().to_string();
    t.delete(&tx)?;
    LogEntry::new(&tx, u, LogAction::DeleteTag { id, name })?;
    tx.commit()?;

    Ok((StatusCode::OK, TAG_DELETED).into_response())
}
