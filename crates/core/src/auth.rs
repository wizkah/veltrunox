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
    #[error("password must have at least {min} characters")]
    PasswordTooShort { min: usize },
    #[error("password must contain a lowercase letter")]
    PasswordNeedsLowercase,
    #[error("password must contain an uppercase letter")]
    PasswordNeedsUppercase,
    #[error("password must contain a digit")]
    PasswordNeedsDigit,
    #[error("password must contain a special character")]
    PasswordNeedsSpecial,
}

impl crate::Refusal for InvalidUser {
    fn reason(&self) -> crate::Reason {
        match self {
            Self::EmptyName => crate::Reason::new(self, "user_name_empty"),
            Self::NameTooLong => crate::Reason::with_args(
                self,
                "user_name_too_long",
                vec![MAX_USER_NAME_CHARS as u64],
            ),
            Self::ControlCharacter => crate::Reason::new(self, "user_name_control_character"),
            Self::PasswordTooShort { min } => {
                crate::Reason::with_args(self, "password_too_short", vec![*min as u64])
            }
            Self::PasswordNeedsLowercase => crate::Reason::new(self, "password_needs_lowercase"),
            Self::PasswordNeedsUppercase => crate::Reason::new(self, "password_needs_uppercase"),
            Self::PasswordNeedsDigit => crate::Reason::new(self, "password_needs_digit"),
            Self::PasswordNeedsSpecial => crate::Reason::new(self, "password_needs_special"),
        }
    }
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

/// How demanding the rules for new passwords are; the admin chooses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PasswordPolicy {
    /// At least 6 characters.
    #[default]
    Standard,
    /// At least 10 characters with a lowercase letter, an uppercase letter, a digit
    /// and a special character.
    Strict,
}

impl PasswordPolicy {
    pub const fn min_chars(self) -> usize {
        match self {
            Self::Standard => 6,
            Self::Strict => 10,
        }
    }

    /// Checks a new password. Passwords set earlier stay valid under any policy.
    pub fn check(self, password: &str) -> Result<(), InvalidUser> {
        let min = self.min_chars();
        if password.chars().count() < min {
            return Err(InvalidUser::PasswordTooShort { min });
        }
        if self == Self::Standard {
            return Ok(());
        }
        let has = |test: fn(&char) -> bool| password.chars().any(|c| test(&c));
        if !has(|c| c.is_lowercase()) {
            return Err(InvalidUser::PasswordNeedsLowercase);
        }
        if !has(|c| c.is_uppercase()) {
            return Err(InvalidUser::PasswordNeedsUppercase);
        }
        if !has(|c| c.is_numeric()) {
            return Err(InvalidUser::PasswordNeedsDigit);
        }
        if !has(|c| !c.is_alphanumeric() && !c.is_whitespace() && !c.is_control()) {
            return Err(InvalidUser::PasswordNeedsSpecial);
        }
        Ok(())
    }
}

/// Identifies an API token of a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ApiTokenId(u64);

impl ApiTokenId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

pub const MAX_TOKEN_NAME_CHARS: usize = 64;

/// The label a user gives an API token, to tell the tokens apart: trimmed and not empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenName(String);

impl TokenName {
    pub fn new(text: &str) -> Result<Self, crate::InvalidValue> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(crate::InvalidValue::EmptyName);
        }
        if trimmed.chars().count() > MAX_TOKEN_NAME_CHARS {
            return Err(crate::InvalidValue::NameTooLong);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// How far an API token reaches at most; what it may do is also limited by the rights of
/// its user. A token never plays, uploads or manages anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenScope {
    /// Reads the library, the bookmarks and the exports.
    ReadOnly,
    /// Reads, and writes bookmarks (edit, replace, import).
    EditBookmarks,
}

impl TokenScope {
    pub const ALL: [Self; 2] = [Self::ReadOnly, Self::EditBookmarks];

    /// Reading needs no capability, so a read-only token holds none.
    pub fn allows(self, capability: Capability) -> bool {
        match self {
            Self::ReadOnly => false,
            Self::EditBookmarks => capability == Capability::EditBookmarks,
        }
    }
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
    fn a_token_name_is_trimmed_and_limited() {
        assert_eq!(TokenName::new(" LLM ").unwrap().as_str(), "LLM");
        assert_eq!(TokenName::new(" "), Err(crate::InvalidValue::EmptyName));
        let long = "x".repeat(MAX_TOKEN_NAME_CHARS + 1);
        assert_eq!(TokenName::new(&long), Err(crate::InvalidValue::NameTooLong));
    }

    #[test]
    fn a_token_never_reaches_beyond_bookmarks() {
        for capability in Capability::ALL {
            assert!(!TokenScope::ReadOnly.allows(capability));
            assert_eq!(
                TokenScope::EditBookmarks.allows(capability),
                capability == Capability::EditBookmarks
            );
        }
    }

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
    fn the_standard_policy_asks_for_six_characters_only() {
        let policy = PasswordPolicy::default();
        assert_eq!(policy, PasswordPolicy::Standard);
        assert_eq!(
            policy.check("12345"),
            Err(InvalidUser::PasswordTooShort { min: 6 })
        );
        assert_eq!(policy.check("123456"), Ok(()));
    }

    #[test]
    fn the_strict_policy_asks_for_length_and_every_kind_of_character() {
        let strict = PasswordPolicy::Strict;
        assert_eq!(
            strict.check("Ab1!xyz"),
            Err(InvalidUser::PasswordTooShort { min: 10 })
        );
        assert_eq!(
            strict.check("ABCDEFG1!2"),
            Err(InvalidUser::PasswordNeedsLowercase)
        );
        assert_eq!(
            strict.check("abcdefg1!2"),
            Err(InvalidUser::PasswordNeedsUppercase)
        );
        assert_eq!(
            strict.check("Abcdefghi!"),
            Err(InvalidUser::PasswordNeedsDigit)
        );
        assert_eq!(
            strict.check("Abcdefghi1"),
            Err(InvalidUser::PasswordNeedsSpecial)
        );
        // A space is not a special character.
        assert_eq!(
            strict.check("Abcdef ghi1"),
            Err(InvalidUser::PasswordNeedsSpecial)
        );
        assert_eq!(strict.check("Abcdefgh1!"), Ok(()));
    }
}
