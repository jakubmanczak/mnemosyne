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
    },
};

pub async fn get_me(h: HeaderMap) -> Result<Response, CompositeError> {
    Ok(Json(User::authenticate(&h)?.required()?).into_response())
}

pub async fn get_by_id(Path(id): Path<Uuid>, h: HeaderMap) -> Result<Response, CompositeError> {
    User::authenticate(&h)?.required()?;
    Ok(Json(User::get_by_id(id)?).into_response())
}
