use axum::{
    Router,
    http::header,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};

mod components;
mod icons;
mod pages;

pub const STYLES_CSS: &str = include_str!("./styles.css");

pub fn web_router() -> Router {
    Router::new()
        .route(
            "/styles.css",
            get(|| async { ([(header::CONTENT_TYPE, "text/css")], STYLES_CSS) }),
        )
        .merge(pages::pages())
}

pub struct RedirectViaError(Redirect);
impl IntoResponse for RedirectViaError {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}
