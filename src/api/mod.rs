use axum::{
    Router,
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::{
    tags::TagError,
    users::{UserError, auth::AuthError, sessions::SessionError},
};

mod auth;
mod sessions;
mod tags;
mod users;

pub fn api_router() -> Router {
    Router::new()
        .route("/api/live", get(async || "Mnemosyne lives"))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/users/me", get(users::get_me))
        .route("/api/users/{id}", get(users::get_by_id))
        .route("/api/users/@{handle}", get(users::get_by_handle))
        .route("/api/sessions/{id}", get(sessions::get_by_id))
        .route("/api/tags/{id}", get(tags::get_by_id))
        .route("/api/tags/#{name}", get(tags::get_by_name))
}

pub struct CompositeError(Response);
impl IntoResponse for CompositeError {
    fn into_response(self) -> Response {
        self.0
    }
}

macro_rules! composite_from {
    ($($t:ty),+ $(,)?) => {
        $(
            impl From<$t> for CompositeError {
                fn from(e: $t) -> Self {
                    CompositeError(e.into_response())
                }
            }
        )+
    };
}
composite_from!(AuthError, UserError, SessionError, TagError);
