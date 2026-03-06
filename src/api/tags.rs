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
    tags::{Tag, TagName},
    users::{
        User,
        auth::{UserAuthRequired, UserAuthenticate},
        permissions::Permission,
    },
};

const CANT_MAKE_TAGS: &str = "You don't have permission to create new tags.";
const CANT_DEL_TAGS: &str = "You don't have permission to delete tags.";
const TAG_DELETED: &str = "Tag deleted successfully.";

pub async fn get_all(headers: HeaderMap) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    Ok(Json(Tag::get_all()?).into_response())
}

pub async fn get_by_id(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    Ok(Json(Tag::get_by_id(id)?).into_response())
}

pub async fn get_by_name(
    Path(name): Path<TagName>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    Ok(Json(Tag::get_by_name(name)?).into_response())
}

#[derive(Deserialize)]
pub struct NewTag {
    name: TagName,
}
pub async fn create(
    headers: HeaderMap,
    Json(form): Json<NewTag>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    if !u.has_permission(Permission::CreateTags)? {
        return Ok((StatusCode::FORBIDDEN, CANT_MAKE_TAGS).into_response());
    }
    Ok(Json(Tag::create(form.name)?).into_response())
}

pub async fn delete(Path(id): Path<Uuid>, headers: HeaderMap) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    if !u.has_permission(Permission::DeleteTags)? {
        return Ok((StatusCode::FORBIDDEN, CANT_DEL_TAGS).into_response());
    }
    Tag::get_by_id(id)?.delete()?;
    Ok((StatusCode::OK, TAG_DELETED).into_response())
}
