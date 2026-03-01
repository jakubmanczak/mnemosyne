use uuid::Uuid;

use crate::persons::{Person, names::Name};

#[allow(unused)]
pub struct QuoteLine {
    pub id: Uuid,
    pub attribution: (Name, Person),
    pub content: String,
}
