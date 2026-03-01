use std::{fmt::Display, hash::Hash, ops::Deref, str::FromStr};

use rusqlite::{
    Result as RusqliteResult,
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct UserHandle(String);

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq, Serialize)]
pub enum UserHandleError {
    #[error("Handle is too short - must be 3 or more characters.")]
    TooShort,
    #[error("Handle is too long - must be 16 or less characters.")]
    TooLong,
    #[error("Handle must consist of ASCII alphanumeric characters only.")]
    NonAsciiAlphanumeric,
}

impl UserHandle {
    pub fn new(input: impl AsRef<str>) -> Result<Self, UserHandleError> {
        let s = input.as_ref();
        UserHandle::validate_str(s)?;
        Ok(UserHandle(s.to_string()))
    }
    pub fn validate_str(str: &str) -> Result<(), UserHandleError> {
        match str.len() {
            ..=2 => return Err(UserHandleError::TooShort),
            17.. => return Err(UserHandleError::TooLong),
            _ => (),
        };
        if str.bytes().any(|c| !c.is_ascii_alphanumeric()) {
            return Err(UserHandleError::NonAsciiAlphanumeric);
        }
        Ok(())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialEq for UserHandle {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}
impl Eq for UserHandle {}
impl Hash for UserHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}

impl AsRef<str> for UserHandle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl Deref for UserHandle {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Display for UserHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl FromStr for UserHandle {
    type Err = UserHandleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate_str(s)?;
        Ok(UserHandle(s.to_string()))
    }
}
impl TryFrom<String> for UserHandle {
    type Error = UserHandleError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate_str(&value)?;
        Ok(UserHandle(value))
    }
}
impl From<UserHandle> for String {
    fn from(value: UserHandle) -> Self {
        value.0
    }
}

impl ToSql for UserHandle {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for UserHandle {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        UserHandle::from_str(value.as_str()?).map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}
