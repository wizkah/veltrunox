//! Signing in and the rights of the signed-in person, as the client core sees them.

use veltrunox_core::{Capability, Role, UserId};

/// What a person types to sign in or to set up the first admin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub user_name: String,
    pub password: String,
}

/// What the server tells anybody before sign-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessStatus {
    /// No user exists yet: the first admin must be set up.
    pub setup_needed: bool,
    pub guest_enabled: bool,
}

/// Who the server takes the client for and what it may do. The interface
/// hides what is not allowed; the server checks every action anyway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rights {
    /// Absent for a guest.
    pub user_id: Option<UserId>,
    pub user_name: Option<String>,
    pub role: Role,
    pub capabilities: Vec<Capability>,
}

impl Rights {
    pub fn allows(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }
}

/// A completed sign-in: the token proves it on later requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedIn {
    pub token: String,
    pub rights: Rights,
}

/// How a client presents itself on the real-time channel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ChannelAccess {
    /// Sends nothing; for servers without sign-in.
    #[default]
    Open,
    Token(String),
    Guest,
}

/// A user account as the admin sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedUser {
    pub id: UserId,
    pub user_name: String,
    pub role: Role,
    pub active: bool,
}

/// A new account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewUser {
    pub user_name: String,
    pub password: String,
    pub role: Role,
}

/// Changes only the fields that are present.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserChange {
    pub role: Option<Role>,
    pub active: Option<bool>,
    pub password: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rights_allow_what_the_server_listed() {
        let rights = Rights {
            user_id: None,
            user_name: None,
            role: Role::Guest,
            capabilities: vec![Capability::PlayControl],
        };
        assert!(rights.allows(Capability::PlayControl));
        assert!(!rights.allows(Capability::ManageUsers));
    }
}
