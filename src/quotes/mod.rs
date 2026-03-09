use axum::{http::StatusCode, response::IntoResponse};
use chrono::{DateTime, FixedOffset};
use rusqlite::OptionalExtension;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    database::{self, DatabaseError},
    persons::Name,
};

#[derive(Serialize)]
pub struct Quote {
    pub id: Uuid,
    pub lines: Vec<QuoteLine>,
    pub timestamp: DateTime<FixedOffset>,
    pub location: Option<String>,
    pub context: Option<String>,
    pub created_by: Uuid,
    pub public: bool,
}

#[derive(Serialize)]
pub struct QuoteLine {
    pub id: Uuid,
    pub attribution: Name,
    pub content: String,
}

#[derive(Debug, thiserror::Error)]
pub enum QuoteError {
    #[error("No quote with ID {0}")]
    NoQuoteWithId(Uuid),
    #[error("A quote must have at least one line")]
    EmptyQuote,
    #[error("{0}")]
    DatabaseError(#[from] DatabaseError),
}

impl Quote {
    pub fn total_count() -> Result<i64, QuoteError> {
        let conn = database::conn()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM quotes", (), |r| r.get(0))?;
        Ok(count)
    }
    pub fn get_by_id(id: Uuid) -> Result<Quote, QuoteError> {
        let conn = database::conn()?;

        let quotemain = conn
            .prepare(
                "SELECT timestamp, location, context, created_by, public FROM quotes WHERE id = ?1",
            )?
            .query_row((id,), |r| {
                Ok((
                    r.get::<_, DateTime<FixedOffset>>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Uuid>(3)?,
                    r.get::<_, bool>(4)?,
                ))
            })
            .optional()?;

        let (timestamp, location, context, created_by, public) = match quotemain {
            Some(data) => data,
            None => return Err(QuoteError::NoQuoteWithId(id)),
        };

        let lines = conn
            .prepare(
                r#"
                SELECT l.id, l.content, n.id, n.is_primary, n.person_id, n.created_by, n.name
                FROM lines AS l JOIN names AS n ON l.name_id = n.id
                WHERE l.quote_id = ?1 ORDER BY l.ordering
                "#,
            )?
            .query_map((id,), |r| {
                Ok(QuoteLine {
                    id: r.get(0)?,
                    content: r.get(1)?,
                    attribution: Name {
                        id: r.get(2)?,
                        is_primary: r.get(3)?,
                        person_id: r.get(4)?,
                        created_by: r.get(5)?,
                        name: r.get(6)?,
                    },
                })
            })?
            .collect::<Result<Vec<QuoteLine>, _>>()?;

        Ok(Quote {
            id,
            lines,
            timestamp,
            location,
            context,
            created_by,
            public,
        })
    }
    pub fn create(
        lines: Vec<(String, Name)>,
        timestamp: DateTime<FixedOffset>,
        context: Option<String>,
        location: Option<String>,
        created_by: Uuid,
        public: bool,
    ) -> Result<Quote, QuoteError> {
        if lines.is_empty() {
            return Err(QuoteError::EmptyQuote);
        }

        let conn = database::conn()?;
        let quote_id = Uuid::now_v7();
        let lines: Vec<(Uuid, String, Name)> = lines
            .into_iter()
            .map(|(c, a)| (Uuid::now_v7(), c, a))
            .collect();

        conn.execute("BEGIN TRANSACTION", ())?;

        let mut quote_stmt = conn.prepare(
            r#"
            INSERT INTO quotes (id, timestamp, location, context, created_by, public)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )?;
        quote_stmt.execute((quote_id, timestamp, &location, &context, created_by, public))?;

        let mut line_stmt = conn.prepare(
            r#"
            INSERT INTO lines (id, quote_id, content, name_id, ordering)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )?;
        for (ordering, (id, content, attr)) in lines.iter().enumerate() {
            line_stmt.execute((id, quote_id, content, attr.id, ordering as i64))?;
        }

        conn.execute("COMMIT", ())?;
        Ok(Quote {
            id: quote_id,
            lines: lines
                .into_iter()
                .map(|(id, content, attribution)| QuoteLine {
                    id,
                    content,
                    attribution,
                })
                .collect(),
            timestamp,
            location,
            context,
            created_by,
            public,
        })
    }
}

impl From<rusqlite::Error> for QuoteError {
    fn from(error: rusqlite::Error) -> Self {
        QuoteError::DatabaseError(DatabaseError::from(error))
    }
}

impl IntoResponse for QuoteError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::DatabaseError(e) => e.into_response(),
            Self::NoQuoteWithId(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            Self::EmptyQuote => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
        }
    }
}
