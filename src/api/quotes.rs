use axum::{
    Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::CompositeError,
    persons::Name,
    quotes::Quote,
    users::{
        User,
        auth::{UserAuthRequired, UserAuthenticate},
    },
};

pub async fn get_by_id(
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, CompositeError> {
    User::authenticate(&headers)?.required()?;
    Ok(Json(Quote::get_by_id(id)?).into_response())
}

#[derive(Deserialize)]
pub struct QuoteLineForm {
    pub content: String,
    pub name_id: Uuid,
}

#[derive(Deserialize)]
pub struct QuoteCreateForm {
    pub lines: Vec<QuoteLineForm>,
    pub timestamp: DateTime<FixedOffset>,
    pub context: Option<String>,
    pub location: Option<String>,
    pub public: bool,
}

pub async fn create(
    headers: HeaderMap,
    Json(form): Json<QuoteCreateForm>,
) -> Result<Response, CompositeError> {
    let u = User::authenticate(&headers)?.required()?;

    let lines = form
        .lines
        .into_iter()
        .map(|l| Ok((l.content, Name::get_by_id(l.name_id)?)))
        .collect::<Result<Vec<(String, Name)>, CompositeError>>()?;

    let q = Quote::create(
        lines,
        form.timestamp,
        form.context,
        form.location,
        u.id,
        form.public,
    )?;

    Ok((StatusCode::CREATED, Json(q)).into_response())
}
