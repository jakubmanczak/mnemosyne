use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::users::handle::UserHandle;

pub mod auth;
pub mod handle;
pub mod sessions;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub handle: UserHandle,
}
