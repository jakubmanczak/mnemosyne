use axum::{
    Router,
    response::{IntoResponse, Response},
    routing::get,
};

use crate::users::{UserError, auth::AuthError, sessions::SessionError};

mod sessions;
mod users;

// TODO: PERMISSIONS FOR ENDPOINTS & ACTIONS
pub fn api_router() -> Router {
    Router::new()
        .route("/api/live", get(async || "Mnemosyne lives"))
        .route("/api/users/me", get(users::get_me))
        .route("/api/users/{id}", get(users::get_by_id))
        .route("/api/users/@{handle}", get(users::get_by_handle))
        .route("/api/sessions/{id}", get(sessions::get_by_id))
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
composite_from!(AuthError, UserError, SessionError);
