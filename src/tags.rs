use std::{fmt::Display, hash::Hash, ops::Deref, str::FromStr};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rusqlite::{
    OptionalExtension, Result as RusqliteResult, ToSql,
    ffi::SQLITE_CONSTRAINT_UNIQUE,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::{self, DatabaseError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: TagName,
}

impl Tag {
    pub fn get_all() -> Result<Vec<Tag>, TagError> {
        Ok(database::conn()?
            .prepare("SELECT id, tagname FROM tags")?
            .query_map((), |r| {
                Ok(Tag {
                    id: r.get(0)?,
                    name: r.get(1)?,
                })
            })?
            .collect::<Result<Vec<Tag>, _>>()?)
    }
    pub fn get_by_id(id: Uuid) -> Result<Tag, TagError> {
        let res = database::conn()?
            .prepare("SELECT tagname FROM tags WHERE id = ?1")?
            .query_one((&id,), |r| {
                Ok(Tag {
                    id,
                    name: r.get(0)?,
                })
            })
            .optional()?;
        match res {
            Some(t) => Ok(t),
            None => Err(TagError::NoTagWithId(id)),
        }
    }
    pub fn get_by_name(name: TagName) -> Result<Tag, TagError> {
        let res = database::conn()?
            .prepare("SELECT id, tagname FROM tags WHERE tagname = ?1")?
            .query_one((&name,), |r| {
                Ok(Tag {
                    id: r.get(0)?,
                    name: r.get(1)?,
                })
            })
            .optional()?;
        match res {
            Some(u) => Ok(u),
            None => Err(TagError::NoTagWithName(name)),
        }
    }
    pub fn create(name: TagName) -> Result<Tag, TagError> {
        let id = Uuid::now_v7();
        database::conn()?
            .prepare("INSERT INTO tags(id, tagname) VALUES (?1, ?2)")?
            .execute((id, &name))?;
        Ok(Tag { id, name })
    }
    pub fn rename(&mut self, name: TagName) -> Result<(), TagError> {
        database::conn()?
            .prepare("UPDATE tags SET tagname = ?1 WHERE id = ?2")?
            .execute((&name, self.id))?;
        self.name = name;
        Ok(())
    }
    pub fn delete(self) -> Result<(), TagError> {
        database::conn()?
            .prepare("DELETE FROM tags WHERE id = ?1")?
            .execute((self.id,))?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("TagNameError: {0}")]
    TagNameError(#[from] TagNameError),
    #[error("No tag found with ID {0}")]
    NoTagWithId(Uuid),
    #[error("No tag found with name {0}")]
    NoTagWithName(TagName),
    #[error("A tag with this name already exists")]
    TagAlreadyExists,
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
}
impl From<rusqlite::Error> for TagError {
    fn from(error: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(e, Some(msg)) = &error
            && e.extended_code == SQLITE_CONSTRAINT_UNIQUE
            && msg.contains("tagname")
        {
            return TagError::TagAlreadyExists;
        }
        TagError::DatabaseError(DatabaseError::from(error))
    }
}
impl IntoResponse for TagError {
    fn into_response(self) -> Response {
        match self {
            Self::DatabaseError(e) => e.into_response(),
            Self::TagAlreadyExists => (StatusCode::CONFLICT, self.to_string()).into_response(),
            Self::TagNameError(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            Self::NoTagWithId(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            Self::NoTagWithName(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct TagName(String);

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq, Serialize)]
pub enum TagNameError {
    #[error("Tag is too short - must be 2 or more characters.")]
    TooShort,
    #[error("Tag is too long - must be 24 or less characters.")]
    TooLong,
    #[error("Tag must consist of ASCII alphanumerics or mid-tag dashes only.")]
    NonDashAsciiAlphanumeric,
    #[error("Tag must not have a leading or trailing dash.")]
    LeadingTrailingDash,
    #[error("Tag must not have consecutive dashes.")]
    ConsecutiveDashes,
}

impl TagName {
    pub fn new(input: impl AsRef<str>) -> Result<Self, TagNameError> {
        let s = input.as_ref();
        TagName::validate_str(s)?;
        Ok(TagName(s.to_string()))
    }
    pub fn validate_str(str: &str) -> Result<(), TagNameError> {
        match str.len() {
            ..2 => return Err(TagNameError::TooShort),
            25.. => return Err(TagNameError::TooLong),
            _ => (),
        };
        if str.bytes().any(|c| !c.is_ascii_alphanumeric() && c != b'-') {
            return Err(TagNameError::NonDashAsciiAlphanumeric);
        }
        if str.starts_with('-') || str.ends_with('-') {
            return Err(TagNameError::LeadingTrailingDash);
        }
        if str
            .as_bytes()
            .windows(2)
            .any(|w| w[0] == b'-' && w[1] == b'-')
        {
            return Err(TagNameError::ConsecutiveDashes);
        }
        Ok(())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialEq for TagName {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}
impl Eq for TagName {}
impl Hash for TagName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}

impl AsRef<str> for TagName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl Deref for TagName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Display for TagName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl FromStr for TagName {
    type Err = TagNameError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate_str(s)?;
        Ok(TagName(s.to_string()))
    }
}
impl TryFrom<String> for TagName {
    type Error = TagNameError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate_str(&value)?;
        Ok(TagName(value))
    }
}
impl From<TagName> for String {
    fn from(value: TagName) -> Self {
        value.0
    }
}

impl ToSql for TagName {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for TagName {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        TagName::from_str(value.as_str()?).map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

#[test]
#[should_panic]
fn tagname_leading_dash_fail() {
    TagName::new("-test").unwrap();
}
#[test]
#[should_panic]
fn tagname_trailing_dash_fail() {
    TagName::new("test-").unwrap();
}
#[test]
#[should_panic]
fn tagname_consecutive_dash_fail() {
    TagName::new("test1--test2").unwrap();
}
#[test]
#[should_panic]
fn tagname_short_fail() {
    TagName::new("x").unwrap();
}
#[test]
fn tagname_short_pass() {
    TagName::new("xd").unwrap();
}
#[test]
#[should_panic]
fn tagname_long_fail() {
    TagName::new("1234567890123456789012345").unwrap();
}
#[test]
fn tagname_long_pass() {
    TagName::new("123456789012345678901234").unwrap();
}
#[test]
#[should_panic]
fn tagname_nondashasciialphanumerics_fail() {
    TagName::new("hate_underscores").unwrap();
}
#[test]
fn tagname_pass() {
    TagName::new("H311-yeah").unwrap();
}
