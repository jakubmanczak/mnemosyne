use axum::{
    Json,
    extract::Path,
    http::HeaderMap,
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
    },
};

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
    User::authenticate(&headers)?.required()?;
    Ok(Json(Tag::create(form.name)?).into_response())
}
