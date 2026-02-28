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
        permissions::Permission,
        sessions::{Session, SessionError},
    },
};

pub async fn get_by_id(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;
    let s = Session::get_by_id(id)?;

    match s.user_id == u.id
        || u.has_permission(Permission::ListOthersSessions)
            .is_ok_and(|v| v)
    {
        true => Ok(Json(s).into_response()),
        false => Err(SessionError::NoSessionWithId(id))?,
    }
}
