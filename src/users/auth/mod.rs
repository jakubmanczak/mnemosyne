use axum::http::HeaderMap;
use rand08::{RngCore, rngs::OsRng};

use crate::users::{User, UserError, sessions::SessionError};

mod implementation;

pub const COOKIE_NAME: &str = "mnemohash";

pub trait UserAuthenticate {
    fn authenticate(headers: &HeaderMap) -> Result<Option<User>, AuthError>;
}
pub trait UserPasswordHashing {
    /// Returns the hashed password as a String
    fn hash_password(passw: &str) -> Result<String, argon2::password_hash::Error>;
    /// Returns whether the password matches the hash
    fn match_hash_password(passw: &str, hash: &str) -> Result<bool, argon2::password_hash::Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Session error: {0}")]
    SessionError(#[from] SessionError),
    #[error("User error: {0}")]
    UserError(#[from] UserError),
    #[error("Invalid authorization header format")]
    InvalidFormat,
    #[error("Invalid base64 encoding")]
    InvalidBase64(#[from] base64::DecodeError),
    #[error("Invalid UTF-8 in credentials")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Argon2 passhash error: {0}")]
    PassHashError(argon2::password_hash::Error),
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum TokenSize {
    /// 5 bytes = 8 chars
    Char8,
    /// 10 bytes = 16 chars
    Char16,
    /// 20 bytes = 32 chars
    Char32,
    /// 40 bytes = 64 chars
    Char64,
}

pub fn generate_token(len: TokenSize) -> String {
    let mut bytes = vec![0u8; len.bytes()];
    let mut rng = OsRng;
    rng.try_fill_bytes(&mut bytes).unwrap();
    base32::encode(base32::Alphabet::Crockford, &bytes)
}
