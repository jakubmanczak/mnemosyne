use axum::{
    Router,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};

use crate::{
    database::DatabaseError,
    persons::PersonError,
    quotes::QuoteError,
    tags::TagError,
    users::{UserError, auth::AuthError, sessions::SessionError},
};

mod auth;
mod persons;
mod quotes;
mod sessions;
mod tags;
mod users;

pub fn api_router() -> Router {
    Router::new()
        .route("/api/live", get(async || "Mnemosyne lives"))
        // auth
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/login-form", post(auth::login_form))
        .route("/api/auth/logout", post(auth::logout))
        // users
        .route("/api/users", get(users::get_all))
        .route("/api/users", post(users::create))
        .route("/api/users/me", get(users::get_me))
        .route("/api/users/{id}", get(users::get_by_id))
        .route("/api/users/@{handle}", get(users::get_by_handle))
        .route("/api/users/{id}/setpassw", post(users::change_password))
        .route("/api/users/{id}/sethandle", post(users::change_handle))
        // sessions
        .route("/api/sessions/{id}", get(sessions::get_by_id))
        .route("/api/sessions/{id}/revoke", post(sessions::revoke_by_id))
        // tags
        .route("/api/tags", get(tags::get_all))
        .route("/api/tags", post(tags::create))
        .route("/api/tags/{id}", get(tags::get_by_id))
        .route("/api/tags/{id}", patch(tags::rename))
        .route("/api/tags/{id}", delete(tags::delete))
        .route("/api/tags/#{name}", get(tags::get_by_name))
        // persons & names
        .route("/api/persons", get(persons::get_all))
        .route("/api/persons", post(persons::create))
        .route("/api/persons/{id}", get(persons::get_by_id))
        .route("/api/persons/{id}/names", get(persons::pid_names))
        .route("/api/persons/{id}/addname", post(persons::add_name))
        .route("/api/names/{id}", get(persons::n_by_id))
        .route("/api/names/{id}/setprimary", post(persons::n_setprimary))
        // quotes
        .route("/api/quotes", post(quotes::create))
        .route("/api/quotes/{id}", get(quotes::get_by_id))
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
composite_from!(
    AuthError,
    UserError,
    SessionError,
    TagError,
    PersonError,
    QuoteError,
    DatabaseError
);
