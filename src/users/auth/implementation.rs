use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
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
    ISE_MSG, database,
    users::{
        User,
        auth::{
            AuthError, COOKIE_NAME, TokenSize, UserAuthRequired, UserAuthenticate,
            UserPasswordHashing,
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
            Self::DatabaseError(e) => {
                eprintln!("[ERROR] Database error occured: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, ISE_MSG.to_string()).into_response()
            }
            Self::PassHashError(e) => {
                eprintln!("[ERROR] A passwordhash error occured: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, ISE_MSG.to_string()).into_response()
            }
        }
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

impl UserPasswordHashing for User {
    fn hash_password(passw: &str) -> Result<String, argon2::password_hash::Error> {
        use rand08::rngs::OsRng as ArgonOsRng;
        let argon = Argon2::default();
        let passw = passw.as_bytes();
        let salt = SaltString::generate(&mut ArgonOsRng);

        Ok(argon.hash_password(passw, &salt)?.to_string())
    }
    fn match_hash_password(passw: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
        let argon = Argon2::default();
        let passw = passw.as_bytes();
        let hash = PasswordHash::try_from(hash)?;
        Ok(argon.verify_password(passw, &hash).is_ok())
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

        let mut basic_auth: Option<&str> = None;
        let mut bearer_auth: Option<&str> = None;
        for header in &auth_values {
            let header = header.trim();
            match AuthScheme::from_header(header) {
                AuthScheme::Basic(creds) => {
                    if basic_auth.is_none() {
                        basic_auth = Some(creds);
                    }
                }
                AuthScheme::Bearer(token) => {
                    if bearer_auth.is_none() {
                        bearer_auth = Some(token);
                    }
                }
                AuthScheme::None => {}
            }
        }

        match (basic_auth, bearer_auth) {
            (Some(creds), _) => authenticate_basic(creds),
            (None, Some(token)) => authenticate_bearer(token),
            _ => Ok(None),
        }
    }
}

fn authenticate_basic(credentials: &str) -> Result<Option<User>, AuthError> {
    let decoded = BASE64_STANDARD.decode(credentials)?;
    let credentials_str = String::from_utf8(decoded)?;

    let Some((username, password)) = credentials_str.split_once(':') else {
        return Err(AuthError::InvalidFormat);
    };
    let conn = database::conn()?;
    let user: Option<(Uuid, Option<String>)> = conn
        .prepare("SELECT id, password FROM users WHERE handle = ?1")?
        .query_row([username], |r| Ok((r.get(0)?, r.get(1)?)))
        .optional()?;

    match user {
        Some((id, Some(passhash))) => match User::match_hash_password(password, &passhash)? {
            true => Ok(Some(User::get_by_id(id)?)),
            false => Err(AuthError::InvalidCredentials),
        },
        _ => Err(AuthError::InvalidCredentials),
    }
}

fn authenticate_bearer(token: &str) -> Result<Option<User>, AuthError> {
    let mut s = Session::get_by_token(token)?;
    if s.is_expired_or_revoked() {
        return Err(AuthError::InvalidCredentials);
    }
    s.prolong()?;
    Ok(Some(User::get_by_id(s.user_id)?))
}
