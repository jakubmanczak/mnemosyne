use axum::{
    Json,
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::users::{
    User,
    auth::{
        AuthError, COOKIE_NAME, SessionAuthRequired, SessionAuthenticate, UserAuthRequired,
        implementation::authenticate_via_credentials,
    },
    sessions::Session,
};

#[derive(Deserialize)]
pub struct LoginForm {
    handle: String,
    password: String,
}

pub async fn login(Json(creds): Json<LoginForm>) -> Result<Response, AuthError> {
    let u = authenticate_via_credentials(&creds.handle, &creds.password)?.required()?;
    let (_, token) = Session::new_for_user(&u)?;

    let secure = match cfg!(debug_assertions) {
        false => "; Secure",
        true => "",
    };
    let cookie = format!(
        "{COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
        Session::DEFAULT_PROLONGATION.num_seconds(),
        secure
    );

    Ok(([(header::SET_COOKIE, cookie)], token).into_response())
}

pub async fn logout(headers: HeaderMap) -> Result<Response, AuthError> {
    let mut s = Session::authenticate(&headers)?.required()?;
    s.revoke(Some(&User::get_by_id(s.user_id)?))?;
    let cookie = format!("{COOKIE_NAME}=revoking; Path=/; HttpOnly; Max-Age=0");
    Ok(([(header::SET_COOKIE, cookie)], "Logged out!").into_response())
}
