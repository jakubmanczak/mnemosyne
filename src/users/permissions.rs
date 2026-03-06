use crate::{database::DatabaseError, users::User};

/// Infradmin and systemuser have all permissions.
pub enum Permission {
    // All Users have the right to observe their own sessions
    ListOthersSessions,
    // All Users have the right to revoke their own sessions
    RevokeOthersSessions,
    // All Users have the right to change their own password
    ChangeOthersPasswords,
    // All Users have the right to change their own handle
    ChangeOthersHandles,
    ManuallyCreateUsers,
    CreateTags,
    RenameTags,
    DeleteTags,
}

impl User {
    pub fn has_permission(
        &self,
        #[allow(unused)] permission: Permission,
    ) -> Result<bool, DatabaseError> {
        // Infradmin and systemuser have all permissions
        if self.is_infradmin() || self.is_systemuser() {
            return Ok(true);
        }

        todo!("Do the permission checking here once permissions are modeled in the DB")
    }
}
