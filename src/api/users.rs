use axum::{
    Json,
    extract::Path,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    api::CompositeError,
    users::{
        User,
        auth::{UserAuthRequired, UserAuthenticate},
        handle::UserHandle,
    },
};

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
