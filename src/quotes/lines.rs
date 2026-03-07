use uuid::Uuid;

use crate::persons::{Name, Person};

#[allow(unused)]
pub struct QuoteLine {
    pub id: Uuid,
    pub attribution: (Name, Person),
    pub content: String,
}
