use axum::{
    Router,
    response::{IntoResponse, Redirect, Response},
};
use tower_http::services::ServeFile;

mod components;
mod icons;
mod pages;

pub fn web_router() -> Router {
    Router::new()
        .route_service("/styles.css", ServeFile::new("src/web/styles.css"))
        .merge(pages::pages())
}

pub struct RedirectViaError(Redirect);
impl IntoResponse for RedirectViaError {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}
