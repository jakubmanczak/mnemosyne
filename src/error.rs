use axum::response::{IntoResponse, Response};

use crate::{
    database::DatabaseError,
    persons::PersonError,
    quotes::QuoteError,
    tags::TagError,
    users::{UserError, auth::AuthError, sessions::SessionError},
    web::RedirectViaError,
};

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
    DatabaseError,
    RedirectViaError,
);

impl From<rusqlite::Error> for CompositeError {
    fn from(e: rusqlite::Error) -> Self {
        CompositeError(DatabaseError::from(e).into_response())
    }
}
