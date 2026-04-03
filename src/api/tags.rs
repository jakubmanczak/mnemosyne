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
    database,
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
    let conn = database::conn()?;
    if !u.has_permission(&conn, Permission::CreateTags)? {
        return Ok((StatusCode::FORBIDDEN, CANT_MAKE_TAGS).into_response());
    }
    Ok(Json(Tag::create(&conn, form.name)?).into_response())
}

pub async fn rename(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(form): Json<TagNameForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    if !u.has_permission(&conn, Permission::RenameTags)? {
        return Ok((StatusCode::FORBIDDEN, CANT_RENAME_TAGS).into_response());
    }
    let mut tag = Tag::get_by_id(&conn, id)?;
    tag.rename(&conn, form.name)?;
    Ok(Json(tag).into_response())
}

pub async fn delete(Path(id): Path<Uuid>, headers: HeaderMap) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let conn = database::conn()?;
    if !u.has_permission(&conn, Permission::DeleteTags)? {
        return Ok((StatusCode::FORBIDDEN, CANT_DEL_TAGS).into_response());
    }
    Tag::get_by_id(&conn, id)?.delete(&conn)?;
    Ok((StatusCode::OK, TAG_DELETED).into_response())
}
