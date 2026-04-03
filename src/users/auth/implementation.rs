use argon2::{PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    http::{
        HeaderMap, StatusCode,
        header::{AUTHORIZATION, COOKIE},
    },
    response::{IntoResponse, Response},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use rusqlite::OptionalExtension;
use uuid::Uuid;

use crate::{
    ISE_MSG,
    database::{self, DatabaseError},
    users::{
        User,
        auth::{
            AuthError, COOKIE_NAME, DUMMY_PASSWORD, DUMMY_PASSWORD_PHC, SHARED_ARGON,
            SessionAuthRequired, SessionAuthenticate, TokenSize, UserAuthRequired,
            UserAuthenticate, UserPasswordHashing,
        },
        sessions::Session,
    },
};

impl TokenSize {
    pub fn bytes(&self) -> usize {
        match self {
            TokenSize::Char8 => 5,
            TokenSize::Char16 => 10,
            TokenSize::Char32 => 20,
            TokenSize::Char64 => 40,
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            Self::InvalidCredentials => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            Self::AuthRequired => (StatusCode::UNAUTHORIZED, self.to_string()).into_response(),
            Self::SessionError(e) => e.into_response(),
            Self::UserError(e) => e.into_response(),
            Self::InvalidFormat => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            Self::InvalidBase64(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            Self::InvalidUtf8(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            Self::DatabaseError(e) => e.into_response(),
            Self::PassHashError(e) => {
                log::error!("[PASSHASH] A passwordhash error occured: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, ISE_MSG.to_string()).into_response()
            }
        }
    }
}
impl From<rusqlite::Error> for AuthError {
    fn from(value: rusqlite::Error) -> Self {
        AuthError::DatabaseError(DatabaseError::from(value))
    }
}

impl UserAuthRequired for Option<User> {
    fn required(self) -> Result<User, AuthError> {
        match self {
            Self::None => Err(AuthError::AuthRequired),
            Self::Some(u) => Ok(u),
        }
    }
}
impl SessionAuthRequired for Option<Session> {
    fn required(self) -> Result<Session, AuthError> {
        match self {
            Self::None => Err(AuthError::AuthRequired),
            Self::Some(s) => Ok(s),
        }
    }
}

impl UserPasswordHashing for User {
    fn hash_password(passw: &str) -> Result<String, argon2::password_hash::Error> {
        use rand08::rngs::OsRng as ArgonOsRng;
        let passw = passw.as_bytes();
        let salt = SaltString::generate(&mut ArgonOsRng);

        Ok(SHARED_ARGON.hash_password(passw, &salt)?.to_string())
    }
    fn match_hash_password(passw: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
        let passw = passw.as_bytes();
        let hash = PasswordHash::try_from(hash)?;
        Ok(SHARED_ARGON.verify_password(passw, &hash).is_ok())
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(err: argon2::password_hash::Error) -> Self {
        AuthError::PassHashError(err)
    }
}

enum AuthScheme<'a> {
    Basic(&'a str),
    Bearer(&'a str),
    None,
}

impl<'a> AuthScheme<'a> {
    fn from_header(header: &'a str) -> Self {
        if let Some(credentials) = header
            .strip_prefix("Basic ")
            .or_else(|| header.strip_prefix("basic "))
        {
            AuthScheme::Basic(credentials)
        } else if let Some(token) = header
            .strip_prefix("Bearer ")
            .or_else(|| header.strip_prefix("bearer "))
        {
            AuthScheme::Bearer(token)
        } else {
            AuthScheme::None
        }
    }
}

impl UserAuthenticate for User {
    fn authenticate(headers: &HeaderMap) -> Result<Option<User>, AuthError> {
        let (basic_auth, bearer_auth) = auth_common(headers);

        match (basic_auth, bearer_auth) {
            (Some(creds), _) => authenticate_basic(&creds),
            (None, Some(token)) => authenticate_bearer(&token),
            _ => Ok(None),
        }
    }
}
impl SessionAuthenticate for Session {
    fn authenticate(headers: &HeaderMap) -> Result<Option<Session>, AuthError> {
        let (_, bearer_auth) = auth_common(headers);
        if let Some(token) = bearer_auth {
            authenticate_bearer_with_session(&token)
        } else {
            Ok(None)
        }
    }
}

fn auth_common(headers: &HeaderMap) -> (Option<String>, Option<String>) {
    let mut auth_values = Vec::new();
    for auth_header in headers.get_all(AUTHORIZATION).iter() {
        if let Ok(s) = auth_header.to_str() {
            auth_values.push(s.to_string());
        }
    }
    for cookie_header in headers.get_all(COOKIE).iter() {
        if let Ok(cookies) = cookie_header.to_str() {
            for cookie in cookies.split(';') {
                let cookie = cookie.trim();
                if let Some(value) = cookie.strip_prefix(&format!("{}=", COOKIE_NAME)) {
                    auth_values.push(format!("Bearer {}", value));
                }
            }
        }
    }
    let mut basic_auth: Option<String> = None;
    let mut bearer_auth: Option<String> = None;
    for header in &auth_values {
        let header = header.trim();
        match AuthScheme::from_header(header) {
            AuthScheme::Basic(creds) => {
                if basic_auth.is_none() {
                    basic_auth = Some(creds.into());
                }
            }
            AuthScheme::Bearer(token) => {
                if bearer_auth.is_none() {
                    bearer_auth = Some(token.into());
                }
            }
            AuthScheme::None => {}
        }
    }
    (basic_auth, bearer_auth)
}

fn authenticate_basic(credentials: &str) -> Result<Option<User>, AuthError> {
    let decoded = BASE64_STANDARD.decode(credentials)?;
    let credentials_str = String::from_utf8(decoded)?;

    let Some((handle, password)) = credentials_str.split_once(':') else {
        return Err(AuthError::InvalidFormat);
    };
    authenticate_via_credentials(handle, password)
}
pub fn authenticate_via_credentials(
    handle: &str,
    password: &str,
) -> Result<Option<User>, AuthError> {
    let conn = database::conn()?;
    let user: Option<(Uuid, Option<String>)> = conn
        .prepare("SELECT id, password FROM users WHERE handle = ?1")?
        .query_row([handle], |r| Ok((r.get(0)?, r.get(1)?)))
        .optional()?;

    match user {
        Some((id, Some(passhash))) => match User::match_hash_password(password, &passhash)? {
            true => Ok(Some(User::get_by_id(&conn, id)?)),
            false => Err(AuthError::InvalidCredentials),
        },
        _ => {
            let _ = User::match_hash_password(DUMMY_PASSWORD, &DUMMY_PASSWORD_PHC)?;
            Err(AuthError::InvalidCredentials)
        }
    }
}

fn authenticate_bearer(token: &str) -> Result<Option<User>, AuthError> {
    let conn = database::conn().map_err(|e| DatabaseError::from(e))?;
    let mut s = Session::get_by_token(&conn, token)?;
    if s.is_expired_or_revoked() {
        return Err(AuthError::InvalidCredentials);
    }
    s.prolong(&conn)?;
    Ok(Some(User::get_by_id(&conn, s.user_id)?))
}
fn authenticate_bearer_with_session(token: &str) -> Result<Option<Session>, AuthError> {
    let conn = database::conn().map_err(|e| DatabaseError::from(e))?;
    let mut s = Session::get_by_token(&conn, token)?;
    if s.is_expired_or_revoked() {
        return Err(AuthError::InvalidCredentials);
    }
    s.prolong(&conn)?;
    Ok(Some(s))
}
