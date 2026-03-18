use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rusqlite::OptionalExtension;
use serde::Serialize;
use uuid::Uuid;

use crate::database::{self, DatabaseError};

#[derive(Serialize)]
pub struct Person {
    pub id: Uuid,
    pub primary_name: String,
    pub created_by: Uuid,
}

#[derive(Serialize)]
pub struct Name {
    pub id: Uuid,
    pub is_primary: bool,
    pub person_id: Uuid,
    pub created_by: Uuid,
    pub name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PersonError {
    #[error("No person found with ID {0}")]
    NoPersonWithId(Uuid),
    #[error("No name found with ID {0}")]
    NoNameWithId(Uuid),
    #[error("A name with this value already exists for this person")]
    NameAlreadyExists,
    #[error("This name is already the primary name")]
    AlreadyPrimary,
    #[error("{0}")]
    DatabaseError(#[from] DatabaseError),
}

impl Person {
    pub fn total_count() -> Result<i64, PersonError> {
        let conn = database::conn()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM persons", (), |r| r.get(0))?;
        Ok(count)
    }
    pub fn get_all() -> Result<Vec<Person>, PersonError> {
        Ok(database::conn()?
            .prepare("SELECT p.id, p.created_by, n.name FROM persons p JOIN names n ON p.id = n.person_id AND n.is_primary = 1")?
            .query_map((), |r| {
                Ok(Person {
                    id: r.get(0)?,
                    created_by: r.get(1)?,
                    primary_name: r.get(2)?,
                })
            })?
            .collect::<Result<Vec<Person>, _>>()?)
    }

    pub fn get_by_id(id: Uuid) -> Result<Person, PersonError> {
        let res = database::conn()?
            .prepare("SELECT p.created_by, n.name FROM persons p JOIN names n ON p.id = n.person_id AND n.is_primary = 1 WHERE p.id = ?1")?
            .query_one((&id,), |r| {
                Ok(Person {
                    id,
                    created_by: r.get(0)?,
                    primary_name: r.get(1)?,
                })
            })
            .optional()?;
        match res {
            Some(p) => Ok(p),
            None => Err(PersonError::NoPersonWithId(id)),
        }
    }

    pub fn get_in_quote_count(&self) -> Result<i64, PersonError> {
        Ok(database::conn()?
            .prepare(
                r#"
            SELECT COUNT(DISTINCT l.quote_id) AS quote_count
            FROM lines l WHERE l.name_id IN (
                SELECT id FROM names WHERE person_id = ?1
            );"#,
            )?
            .query_one((self.id,), |r| Ok(r.get(0)?))?)
    }

    pub fn get_all_names(&self) -> Result<Vec<Name>, PersonError> {
        Ok(database::conn()?
            .prepare("SELECT id, is_primary, person_id, created_by, name FROM names WHERE person_id = ?1")?
            .query_map((&self.id,), |r| {
                Ok(Name {
                    id: r.get(0)?,
                    is_primary: r.get(1)?,
                    person_id: r.get(2)?,
                    created_by: r.get(3)?,
                    name: r.get(4)?,
                })
            })?
            .collect::<Result<Vec<Name>, _>>()?)
    }

    pub fn add_name(&self, name: String, created_by: Uuid) -> Result<Name, PersonError> {
        let id = Uuid::now_v7();
        database::conn()?
            .prepare("INSERT INTO names VALUES (?1, ?2, ?3, ?4, ?5)")?
            .execute((id, 0, self.id, created_by, &name))?;
        Ok(Name {
            id,
            is_primary: false,
            person_id: self.id,
            created_by,
            name,
        })
    }

    pub fn create(primary_name: String, created_by: Uuid) -> Result<Person, PersonError> {
        let person_id = Uuid::now_v7();
        let name_id = Uuid::now_v7();

        let conn = database::conn()?;
        conn.execute("BEGIN TRANSACTION", ())?;

        conn.prepare("INSERT INTO persons(id, created_by) VALUES (?1, ?2)")?
            .execute((person_id, created_by))?;
        conn.prepare("INSERT INTO names VALUES (?1, ?2, ?3, ?4, ?5)")?
            .execute((name_id, 1, person_id, created_by, &primary_name))?;
        conn.execute("COMMIT", ())?;

        Ok(Person {
            id: person_id,
            primary_name,
            created_by,
        })
    }
}

impl Name {
    pub fn get_by_id(id: Uuid) -> Result<Name, PersonError> {
        let res = database::conn()?
            .prepare("SELECT id, is_primary, person_id, created_by, name FROM names WHERE id = ?1")?
            .query_one((&id,), |r| {
                Ok(Name {
                    id: r.get(0)?,
                    is_primary: r.get(1)?,
                    person_id: r.get(2)?,
                    created_by: r.get(3)?,
                    name: r.get(4)?,
                })
            })
            .optional()?;
        match res {
            Some(n) => Ok(n),
            None => Err(PersonError::NoNameWithId(id)),
        }
    }
    pub fn set_primary(&mut self) -> Result<(), PersonError> {
        if self.is_primary {
            return Err(PersonError::AlreadyPrimary);
        }

        let conn = database::conn()?;
        conn.execute("BEGIN TRANSACTION", ())?;

        conn.prepare("UPDATE names SET is_primary = 0 WHERE person_id = ?1 AND is_primary = 1")?
            .execute((&self.person_id,))?;
        conn.prepare("UPDATE names SET is_primary = 1 WHERE id = ?1")?
            .execute((&self.id,))?;

        conn.execute("COMMIT", ())?;
        self.is_primary = true;
        Ok(())
    }
}

impl From<rusqlite::Error> for PersonError {
    fn from(error: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(e, Some(msg)) = &error
            && e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
            && msg.contains("name")
        {
            return PersonError::NameAlreadyExists;
        }
        PersonError::DatabaseError(DatabaseError::from(error))
    }
}

impl IntoResponse for PersonError {
    fn into_response(self) -> Response {
        match self {
            Self::DatabaseError(e) => e.into_response(),
            Self::NoPersonWithId(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            Self::NoNameWithId(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            Self::NameAlreadyExists => (StatusCode::CONFLICT, self.to_string()).into_response(),
            Self::AlreadyPrimary => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
        }
    }
}
