use crate::users::User;

/// Infradmin and systemuser have all permissions.
pub enum Permission {
    // All Users have the right to observe their own sessions
    ListOthersSessions,
}

impl User {
    pub fn has_permission(&self, permission: Permission) -> Result<bool, rusqlite::Error> {
        if self.is_infradmin() || self.is_systemuser() {
            return Ok(true);
        }

        todo!("Do the permission checking here once permissions are modeled in the DB")
    }
}
