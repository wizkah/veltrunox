//! Use cases: API tokens, the long-lived credentials of scripts and LLMs.

use crate::accounts::Identity;
use crate::ports::{
    ApiTokenRecord, ApiTokenRepository, SessionTokens, StorageError, UserRecord, WallClock,
};
use std::sync::Arc;
use veltrunox_core::{ApiTokenId, InvalidValue, Reason, Refusal, TokenName, TokenScope, UserId};

/// Every API token starts with this, so that a token is recognised without a lookup.
pub const API_TOKEN_PREFIX: &str = "vxt_";

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error(transparent)]
    Invalid(#[from] InvalidValue),
    #[error("API token not found")]
    NotFound,
    /// Guests have no account, and a token cannot make tokens.
    #[error("not allowed")]
    Forbidden,
    #[error(transparent)]
    Storage(#[from] StorageError),
}

impl Refusal for TokenError {
    fn reason(&self) -> Reason {
        match self {
            Self::Invalid(error) => error.reason(),
            Self::NotFound => Reason::new(self, "token_not_found"),
            Self::Forbidden => Reason::new(self, "not_allowed"),
            Self::Storage(_) => Reason::new(self, "internal_error"),
        }
    }
}

/// A new token: shown once, only its digest is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedToken {
    pub record: ApiTokenRecord,
    pub token: String,
}

pub struct ApiTokenService {
    repository: Arc<dyn ApiTokenRepository>,
    tokens: Arc<dyn SessionTokens>,
    clock: Arc<dyn WallClock>,
}

impl ApiTokenService {
    pub fn new(
        repository: Arc<dyn ApiTokenRepository>,
        tokens: Arc<dyn SessionTokens>,
        clock: Arc<dyn WallClock>,
    ) -> Self {
        Self {
            repository,
            tokens,
            clock,
        }
    }

    /// Makes a token for the signed-in user. Only a person who signed in with a password may
    /// do this: not a guest, and not a token, which would hand out reach it may not have.
    pub fn create(
        &self,
        actor: &Identity,
        name: &str,
        scope: TokenScope,
    ) -> Result<CreatedToken, TokenError> {
        let user = account_of(actor)?;
        let name = TokenName::new(name)?;
        let token = format!("{API_TOKEN_PREFIX}{}", self.tokens.issue());
        let record = self.repository.create(
            user,
            &name,
            scope,
            &self.tokens.digest(&token),
            self.clock.unix_seconds(),
        )?;
        Ok(CreatedToken { record, token })
    }

    /// The tokens of the signed-in user.
    pub fn list(&self, actor: &Identity) -> Result<Vec<ApiTokenRecord>, TokenError> {
        Ok(self.repository.list_for_user(account_of(actor)?)?)
    }

    /// Removes one of the user's own tokens.
    pub fn revoke(&self, actor: &Identity, id: ApiTokenId) -> Result<(), TokenError> {
        if self.repository.revoke(account_of(actor)?, id)? {
            Ok(())
        } else {
            Err(TokenError::NotFound)
        }
    }

    /// `true` for text that has the shape of an API token.
    pub fn is_api_token(token: &str) -> bool {
        token.starts_with(API_TOKEN_PREFIX)
    }

    /// The identity behind a token; `None` if it is unknown or its user is deactivated.
    pub fn authenticate(&self, token: &str) -> Result<Option<Identity>, TokenError> {
        let found = self
            .repository
            .user_of(&self.tokens.digest(token), self.clock.unix_seconds())?;
        Ok(found
            .map(|(record, scope): (UserRecord, TokenScope)| Identity::of_token(&record, scope)))
    }
}

fn account_of(actor: &Identity) -> Result<UserId, TokenError> {
    match (&actor.user, actor.scope) {
        (Some((id, _)), None) => Ok(*id),
        _ => Err(TokenError::Forbidden),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{CountingTokens, FakeWallClock};
    use std::sync::Mutex;
    use veltrunox_core::{Capability, Role, UserName};

    /// Tokens in memory, all of them belonging to the users the test names.
    #[derive(Default)]
    struct FakeTokens(Mutex<Vec<(ApiTokenRecord, String)>>);

    impl FakeTokens {
        fn owner(user: UserId) -> UserRecord {
            UserRecord {
                id: user,
                name: UserName::new("ann").unwrap(),
                role: Role::User,
                password_hash: String::new(),
                active: true,
            }
        }
    }

    impl ApiTokenRepository for FakeTokens {
        fn create(
            &self,
            user: UserId,
            name: &TokenName,
            scope: TokenScope,
            digest: &str,
            now: u64,
        ) -> Result<ApiTokenRecord, StorageError> {
            let mut stored = self.0.lock().unwrap();
            let record = ApiTokenRecord {
                id: ApiTokenId::new(stored.len() as u64 + 1),
                user,
                name: name.clone(),
                scope,
                created_at: now,
                last_used: None,
            };
            stored.push((record.clone(), digest.to_owned()));
            Ok(record)
        }

        fn list_for_user(&self, user: UserId) -> Result<Vec<ApiTokenRecord>, StorageError> {
            let stored = self.0.lock().unwrap();
            Ok(stored
                .iter()
                .filter(|(record, _)| record.user == user)
                .map(|(record, _)| record.clone())
                .collect())
        }

        fn user_of(
            &self,
            digest: &str,
            _now: u64,
        ) -> Result<Option<(UserRecord, TokenScope)>, StorageError> {
            let stored = self.0.lock().unwrap();
            Ok(stored
                .iter()
                .find(|(_, stored_digest)| stored_digest == digest)
                .map(|(record, _)| (Self::owner(record.user), record.scope)))
        }

        fn revoke(&self, user: UserId, id: ApiTokenId) -> Result<bool, StorageError> {
            let mut stored = self.0.lock().unwrap();
            let before = stored.len();
            stored.retain(|(record, _)| !(record.user == user && record.id == id));
            Ok(stored.len() != before)
        }
    }

    fn service() -> ApiTokenService {
        ApiTokenService::new(
            Arc::new(FakeTokens::default()),
            Arc::new(CountingTokens::default()),
            Arc::new(FakeWallClock::default()),
        )
    }

    fn account(id: u64) -> Identity {
        Identity {
            user: Some((UserId::new(id), UserName::new("ann").unwrap())),
            role: Role::User,
            scope: None,
        }
    }

    #[test]
    fn a_token_is_shown_once_and_recognised_by_its_prefix() {
        let service = service();
        let created = service
            .create(&account(1), " LLM ", TokenScope::EditBookmarks)
            .unwrap();
        assert!(ApiTokenService::is_api_token(&created.token));
        assert_eq!(created.record.name.as_str(), "LLM");
        let identity = service.authenticate(&created.token).unwrap().unwrap();
        assert_eq!(identity.scope, Some(TokenScope::EditBookmarks));
        assert!(service.authenticate("vxt_unknown").unwrap().is_none());
    }

    #[test]
    fn a_token_reaches_only_what_user_and_scope_share() {
        let service = service();
        let read = service
            .create(&account(1), "read", TokenScope::ReadOnly)
            .unwrap();
        let edit = service
            .create(&account(1), "edit", TokenScope::EditBookmarks)
            .unwrap();
        let read = service.authenticate(&read.token).unwrap().unwrap();
        let edit = service.authenticate(&edit.token).unwrap().unwrap();
        assert!(read.capabilities().is_empty());
        assert_eq!(edit.capabilities(), vec![Capability::EditBookmarks]);
        // The user may upload, the token may not.
        assert!(account(1).allows(Capability::Upload));
        assert!(!edit.allows(Capability::Upload));
    }

    #[test]
    fn guests_and_tokens_cannot_make_tokens() {
        let service = service();
        let created = service
            .create(&account(1), "edit", TokenScope::EditBookmarks)
            .unwrap();
        let by_token = service.authenticate(&created.token).unwrap().unwrap();
        for actor in [Identity::guest(), by_token] {
            assert!(matches!(
                service.create(&actor, "more", TokenScope::ReadOnly),
                Err(TokenError::Forbidden)
            ));
            assert!(matches!(service.list(&actor), Err(TokenError::Forbidden)));
        }
    }

    #[test]
    fn users_see_and_revoke_only_their_own_tokens() {
        let service = service();
        let mine = service
            .create(&account(1), "mine", TokenScope::ReadOnly)
            .unwrap();
        service
            .create(&account(2), "theirs", TokenScope::ReadOnly)
            .unwrap();
        assert_eq!(service.list(&account(1)).unwrap().len(), 1);
        assert!(matches!(
            service.revoke(&account(2), mine.record.id),
            Err(TokenError::NotFound)
        ));
        service.revoke(&account(1), mine.record.id).unwrap();
        assert!(service.authenticate(&mine.token).unwrap().is_none());
        assert!(matches!(
            service.revoke(&account(1), mine.record.id),
            Err(TokenError::NotFound)
        ));
    }

    #[test]
    fn a_blank_name_is_refused() {
        assert!(matches!(
            service().create(&account(1), "  ", TokenScope::ReadOnly),
            Err(TokenError::Invalid(InvalidValue::EmptyName))
        ));
    }
}
