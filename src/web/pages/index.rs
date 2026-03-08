use axum::response::{IntoResponse, Redirect, Response};

use crate::users::auth::AuthError;

pub async fn page() -> Result<Response, AuthError> {
    Ok(Redirect::to("/dashboard").into_response())
}
