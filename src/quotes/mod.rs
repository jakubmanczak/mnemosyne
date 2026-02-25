use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::quotes::lines::QuoteLine;

pub mod lines;

pub struct Quote {
    pub id: Uuid,
    pub lines: Vec<QuoteLine>,
    pub timestamp: DateTime<Utc>,
    pub location: Option<String>,
    pub context: Option<String>,
    pub created_by: Uuid,
    pub public: bool,
}
