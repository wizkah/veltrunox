use std::fmt;

/// Identifies a user account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(u64);

impl UserId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

const MAX_USER_NAME_CHARS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidUser {
    #[error("user name must not be empty")]
    EmptyName,
    #[error("user name must have at most {MAX_USER_NAME_CHARS} characters")]
    NameTooLong,
    #[error("user name must not contain control characters")]
    ControlCharacter,
    #[error("password must have at least {MIN_PASSWORD_CHARS} characters")]
    PasswordTooShort,
}

/// A trimmed, non-empty user name. Names are compared without regard to case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserName(String);

impl UserName {
    pub fn new(text: &str) -> Result<Self, InvalidUser> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(InvalidUser::EmptyName);
        }
        if trimmed.chars().count() > MAX_USER_NAME_CHARS {
            return Err(InvalidUser::NameTooLong);
        }
        if trimmed.chars().any(char::is_control) {
            return Err(InvalidUser::ControlCharacter);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The form in which names are compared and stored for lookup.
    pub fn key(&self) -> String {
        self.0.to_lowercase()
    }
}

pub const MIN_PASSWORD_CHARS: usize = 10;

/// Checks the only rule for passwords: a minimum length.
pub fn check_password(password: &str) -> Result<(), InvalidUser> {
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(InvalidUser::PasswordTooShort);
    }
    Ok(())
}

/// What a person may be. The guest exists only while guest access is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    User,
    Guest,
}

/// One kind of action the server authorises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Position, run/pause, speed, tuning, key, volume, play range and mode.
    PlayControl,
    EditBookmarks,
    Upload,
    /// Rename, describe, delete and re-analyse recordings.
    ManageRecordings,
    /// Users and the guest switch.
    ManageUsers,
}

impl Capability {
    pub const ALL: [Self; 5] = [
        Self::PlayControl,
        Self::EditBookmarks,
        Self::Upload,
        Self::ManageRecordings,
        Self::ManageUsers,
    ];
}

impl Role {
    pub fn allows(self, capability: Capability) -> bool {
        match self {
            Self::Admin => true,
            Self::User | Self::Guest => matches!(
                capability,
                Capability::PlayControl | Capability::EditBookmarks | Capability::Upload
            ),
        }
    }

    pub fn capabilities(self) -> Vec<Capability> {
        Capability::ALL
            .into_iter()
            .filter(|capability| self.allows(*capability))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_and_guest_have_the_same_rights() {
        assert_eq!(Role::User.capabilities(), Role::Guest.capabilities());
    }

    #[test]
    fn only_the_admin_manages() {
        for capability in [Capability::ManageUsers, Capability::ManageRecordings] {
            assert!(Role::Admin.allows(capability));
            assert!(!Role::User.allows(capability));
            assert!(!Role::Guest.allows(capability));
        }
    }

    #[test]
    fn user_may_play_edit_bookmarks_and_upload() {
        for capability in [
            Capability::PlayControl,
            Capability::EditBookmarks,
            Capability::Upload,
        ] {
            assert!(Role::User.allows(capability));
        }
    }

    #[test]
    fn names_are_trimmed_and_compared_without_case() {
        let name = UserName::new("  Ada ").unwrap();
        assert_eq!(name.as_str(), "Ada");
        assert_eq!(name.key(), UserName::new("ADA").unwrap().key());
    }

    #[test]
    fn bad_names_are_rejected() {
        assert_eq!(UserName::new("  "), Err(InvalidUser::EmptyName));
        assert_eq!(UserName::new("a\nb"), Err(InvalidUser::ControlCharacter));
        assert_eq!(
            UserName::new(&"x".repeat(65)),
            Err(InvalidUser::NameTooLong)
        );
    }

    #[test]
    fn short_passwords_are_rejected() {
        assert_eq!(
            check_password("123456789"),
            Err(InvalidUser::PasswordTooShort)
        );
        assert_eq!(check_password("1234567890"), Ok(()));
    }
}
