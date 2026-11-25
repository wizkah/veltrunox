//! Wire types for sign-in, rights and user administration.

use serde::{Deserialize, Serialize};
use veltrunox_core::{Capability, PasswordPolicy, Role, TokenScope};

pub const STATUS_PATH: &str = "/api/v1/status";
/// Creates the first admin (body: `CredentialsDto`, answer: `SessionDto`); only while no user exists.
pub const SETUP_PATH: &str = "/api/v1/setup";
/// Signs in (body: `CredentialsDto`, answer: `SessionDto`).
pub const LOGIN_PATH: &str = "/api/v1/login";
pub const LOGOUT_PATH: &str = "/api/v1/logout";
/// Who the caller is and what they may do (`IdentityDto`).
pub const ME_PATH: &str = "/api/v1/me";
/// All users (admin) and creation of one (`NewUserDto`).
pub const USERS_PATH: &str = "/api/v1/users";
/// One user; `{id}` is the user id (`UpdateUserDto`).
pub const USER_PATH: &str = "/api/v1/users/{id}";
/// The guest switch (`GuestAccessDto`).
pub const GUEST_ACCESS_PATH: &str = "/api/v1/guest-access";
/// The rules for new passwords (`PasswordPolicyDto`); changing them is for admins.
pub const PASSWORD_POLICY_PATH: &str = "/api/v1/password-policy";

/// The API tokens of the signed-in user (`GET`) and creation of one (`POST`, `NewApiTokenDto`).
pub const TOKENS_PATH: &str = "/api/v1/tokens";
/// One API token; `{id}` is the token id (`DELETE` revokes it).
pub const TOKEN_PATH: &str = "/api/v1/tokens/{id}";

/// How far an API token reaches at most.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenScopeDto {
    ReadOnly,
    EditBookmarks,
}

pub fn token_scope_to_dto(scope: TokenScope) -> TokenScopeDto {
    match scope {
        TokenScope::ReadOnly => TokenScopeDto::ReadOnly,
        TokenScope::EditBookmarks => TokenScopeDto::EditBookmarks,
    }
}

pub fn token_scope_from_dto(scope: TokenScopeDto) -> TokenScope {
    match scope {
        TokenScopeDto::ReadOnly => TokenScope::ReadOnly,
        TokenScopeDto::EditBookmarks => TokenScope::EditBookmarks,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewApiTokenDto {
    pub name: String,
    pub scope: TokenScopeDto,
}

/// An API token as listed; the token itself is not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiTokenDto {
    pub id: u64,
    pub name: String,
    pub scope: TokenScopeDto,
    /// Unix seconds.
    pub created_at: u64,
    /// Unix seconds; absent until the token was used.
    pub last_used: Option<u64>,
}

/// Answer to the creation of a token: the only time its text is shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedApiTokenDto {
    pub token: String,
    pub api_token: ApiTokenDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiTokenListDto {
    pub tokens: Vec<ApiTokenDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleDto {
    Admin,
    User,
    Guest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDto {
    PlayControl,
    EditBookmarks,
    Upload,
    ManageRecordings,
    ManageUsers,
}

pub fn role_to_dto(role: Role) -> RoleDto {
    match role {
        Role::Admin => RoleDto::Admin,
        Role::User => RoleDto::User,
        Role::Guest => RoleDto::Guest,
    }
}

pub fn role_from_dto(role: RoleDto) -> Role {
    match role {
        RoleDto::Admin => Role::Admin,
        RoleDto::User => Role::User,
        RoleDto::Guest => Role::Guest,
    }
}

pub fn capability_to_dto(capability: Capability) -> CapabilityDto {
    match capability {
        Capability::PlayControl => CapabilityDto::PlayControl,
        Capability::EditBookmarks => CapabilityDto::EditBookmarks,
        Capability::Upload => CapabilityDto::Upload,
        Capability::ManageRecordings => CapabilityDto::ManageRecordings,
        Capability::ManageUsers => CapabilityDto::ManageUsers,
    }
}

pub fn capability_from_dto(capability: CapabilityDto) -> Capability {
    match capability {
        CapabilityDto::PlayControl => Capability::PlayControl,
        CapabilityDto::EditBookmarks => Capability::EditBookmarks,
        CapabilityDto::Upload => Capability::Upload,
        CapabilityDto::ManageRecordings => Capability::ManageRecordings,
        CapabilityDto::ManageUsers => Capability::ManageUsers,
    }
}

/// What a client may learn without signing in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusDto {
    pub protocol_version: u32,
    /// No user exists yet: the first admin must be set up.
    pub setup_needed: bool,
    pub guest_enabled: bool,
    pub password_policy: PasswordPolicyDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PasswordPolicyDto {
    /// At least 6 characters.
    #[default]
    Standard,
    /// At least 10 characters with lowercase, uppercase, digit and special character.
    Strict,
}

pub fn password_policy_to_dto(policy: PasswordPolicy) -> PasswordPolicyDto {
    match policy {
        PasswordPolicy::Standard => PasswordPolicyDto::Standard,
        PasswordPolicy::Strict => PasswordPolicyDto::Strict,
    }
}

pub fn password_policy_from_dto(policy: PasswordPolicyDto) -> PasswordPolicy {
    match policy {
        PasswordPolicyDto::Standard => PasswordPolicy::Standard,
        PasswordPolicyDto::Strict => PasswordPolicy::Strict,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetPasswordPolicyDto {
    pub policy: PasswordPolicyDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialsDto {
    pub user_name: String,
    pub password: String,
}

/// The effective rights of the caller; the interface hides what is not allowed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityDto {
    /// Absent for a guest.
    pub user_id: Option<u64>,
    pub user_name: Option<String>,
    pub role: RoleDto,
    pub capabilities: Vec<CapabilityDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDto {
    /// Sent as `Authorization: Bearer <token>`.
    pub token: String,
    pub identity: IdentityDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserDto {
    pub id: u64,
    pub user_name: String,
    pub role: RoleDto,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewUserDto {
    pub user_name: String,
    pub password: String,
    pub role: RoleDto,
}

/// Changes only the fields that are present.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct UpdateUserDto {
    pub role: Option<RoleDto>,
    pub active: Option<bool>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestAccessDto {
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_and_capabilities_have_stable_names() {
        assert_eq!(
            serde_json::to_string(&RoleDto::Admin).unwrap(),
            r#""admin""#
        );
        assert_eq!(
            serde_json::to_string(&CapabilityDto::ManageUsers).unwrap(),
            r#""manage_users""#
        );
    }

    #[test]
    fn every_capability_maps() {
        for capability in Capability::ALL {
            assert_eq!(
                capability_from_dto(capability_to_dto(capability)),
                capability
            );
        }
        for role in [Role::Admin, Role::User, Role::Guest] {
            assert_eq!(role_from_dto(role_to_dto(role)), role);
        }
    }
}
