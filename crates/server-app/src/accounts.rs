//! Use cases: accounts, login sessions, guest access and the rights check.

use crate::ports::{
    HashFailure, PasswordHasher, SessionRepository, SessionTokens, StorageError, UserRecord,
    UserRepository, UserStoreError, WallClock,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use veltrunox_core::{
    Capability, InvalidUser, PasswordPolicy, Reason, Refusal, Role, TokenScope, UserId, UserName,
};

/// A session unused for this long is gone (30 days).
const SESSION_MAX_IDLE_SECONDS: u64 = 30 * 24 * 60 * 60;
/// Failed logins of one name that are free of delay.
const FREE_ATTEMPTS: u32 = 3;
const MAX_DELAY_SECONDS: u64 = 60;

/// Who is acting. A guest has a role but no account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub user: Option<(UserId, UserName)>,
    pub role: Role,
    /// Set when the caller used an API token: what the token reaches, at most.
    pub scope: Option<TokenScope>,
}

impl Identity {
    /// The role's rights, cut down to what an API token reaches.
    pub fn allows(&self, capability: Capability) -> bool {
        self.role.allows(capability) && self.scope.is_none_or(|scope| scope.allows(capability))
    }

    pub fn capabilities(&self) -> Vec<Capability> {
        Capability::ALL
            .into_iter()
            .filter(|capability| self.allows(*capability))
            .collect()
    }

    pub fn guest() -> Self {
        Self {
            user: None,
            role: Role::Guest,
            scope: None,
        }
    }

    /// An account signed in with its password.
    pub fn of(record: &UserRecord) -> Self {
        Self {
            user: Some((record.id, record.name.clone())),
            role: record.role,
            scope: None,
        }
    }

    /// An account acting through an API token of the given reach.
    pub fn of_token(record: &UserRecord, scope: TokenScope) -> Self {
        Self {
            scope: Some(scope),
            ..Self::of(record)
        }
    }
}

/// A login: the token is shown once, the server keeps only its digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub token: String,
    pub identity: Identity,
}

/// What a client may know before it signs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessStatus {
    pub setup_needed: bool,
    pub guest_enabled: bool,
    pub password_policy: PasswordPolicy,
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error(transparent)]
    Invalid(#[from] InvalidUser),
    #[error("a user with this name exists already")]
    NameTaken,
    #[error("user not found")]
    NotFound,
    #[error("the first admin exists already")]
    SetupDone,
    #[error("wrong user name or password")]
    BadCredentials,
    #[error("the current password is wrong")]
    WrongCurrentPassword,
    #[error("too many failed logins; try again in {retry_after_seconds} seconds")]
    TooManyAttempts { retry_after_seconds: u64 },
    #[error("at least one active admin must remain")]
    LastAdmin,
    #[error("guests are not a role of an account")]
    GuestAccount,
    #[error("not allowed")]
    Forbidden,
    #[error("guest access is off")]
    GuestAccessOff,
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Hash(#[from] HashFailure),
}

impl Refusal for AccountError {
    fn reason(&self) -> Reason {
        match self {
            Self::Invalid(error) => error.reason(),
            Self::NameTaken => Reason::new(self, "user_name_taken"),
            Self::NotFound => Reason::new(self, "user_not_found"),
            Self::SetupDone => Reason::new(self, "setup_done"),
            Self::BadCredentials => Reason::new(self, "bad_credentials"),
            Self::WrongCurrentPassword => Reason::new(self, "wrong_current_password"),
            Self::TooManyAttempts {
                retry_after_seconds,
            } => Reason::with_args(self, "too_many_attempts", vec![*retry_after_seconds]),
            Self::LastAdmin => Reason::new(self, "last_admin"),
            Self::GuestAccount => Reason::new(self, "guest_account"),
            Self::Forbidden => Reason::new(self, "not_allowed"),
            Self::GuestAccessOff => Reason::new(self, "guest_access_off"),
            Self::Storage(_) | Self::Hash(_) => Reason::new(self, "internal_error"),
        }
    }
}

impl From<UserStoreError> for AccountError {
    fn from(error: UserStoreError) -> Self {
        match error {
            UserStoreError::NameTaken => Self::NameTaken,
            UserStoreError::NotFound => Self::NotFound,
            UserStoreError::Storage(error) => Self::Storage(error),
        }
    }
}

struct Failures {
    count: u32,
    blocked_until: u64,
}

pub struct AccountService {
    users: Arc<dyn UserRepository>,
    sessions: Arc<dyn SessionRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn SessionTokens>,
    clock: Arc<dyn WallClock>,
    failures: Mutex<HashMap<String, Failures>>,
    /// Checked for unknown names so that their timing does not differ.
    decoy_hash: String,
}

impl AccountService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        sessions: Arc<dyn SessionRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn SessionTokens>,
        clock: Arc<dyn WallClock>,
    ) -> Result<Self, AccountError> {
        let decoy_hash = hasher.hash(&tokens.issue())?;
        Ok(Self {
            users,
            sessions,
            hasher,
            tokens,
            clock,
            failures: Mutex::new(HashMap::new()),
            decoy_hash,
        })
    }

    pub fn status(&self) -> Result<AccessStatus, AccountError> {
        Ok(AccessStatus {
            setup_needed: self.users.count()? == 0,
            guest_enabled: self.users.guest_enabled()?,
            password_policy: self.users.password_policy()?,
        })
    }

    /// Creates the first admin and signs them in. Only works while no user exists.
    pub fn setup(&self, name: &str, password: &str) -> Result<Session, AccountError> {
        let name = UserName::new(name)?;
        self.users.password_policy()?.check(password)?;
        let hash = self.hasher.hash(password)?;
        match self.users.create_first_admin(&name, &hash)? {
            Some(record) => self.open_session(&record),
            None => Err(AccountError::SetupDone),
        }
    }

    pub fn login(&self, name: &str, password: &str) -> Result<Session, AccountError> {
        let key = UserName::new(name)
            .map(|name| name.key())
            .map_err(|_| AccountError::BadCredentials)?;
        self.ensure_not_blocked(&key)?;
        let record = self.users.find_by_name(&key)?;
        let hash = record
            .as_ref()
            .map_or(self.decoy_hash.as_str(), |record| &record.password_hash);
        let correct = self.hasher.verify(password, hash);
        match record {
            Some(record) if correct && record.active => {
                self.clear_failures(&key);
                self.open_session(&record)
            }
            _ => {
                self.note_failure(&key);
                Err(AccountError::BadCredentials)
            }
        }
    }

    pub fn logout(&self, token: &str) -> Result<(), AccountError> {
        Ok(self.sessions.end(&self.tokens.digest(token))?)
    }

    /// The identity of a token; `None` if it is unknown, expired or of a deactivated user.
    pub fn authenticate(&self, token: &str) -> Result<Option<Identity>, AccountError> {
        let found = self.sessions.user_of(
            &self.tokens.digest(token),
            self.clock.unix_seconds(),
            SESSION_MAX_IDLE_SECONDS,
        )?;
        Ok(found.map(|record| Identity::of(&record)))
    }

    /// The guest identity while the admin allows guests.
    pub fn guest(&self) -> Result<Identity, AccountError> {
        if self.users.guest_enabled()? {
            Ok(Identity::guest())
        } else {
            Err(AccountError::GuestAccessOff)
        }
    }

    pub fn list_users(&self, actor: &Identity) -> Result<Vec<UserRecord>, AccountError> {
        require_manage_users(actor)?;
        Ok(self.users.list()?)
    }

    pub fn create_user(
        &self,
        actor: &Identity,
        name: &str,
        password: &str,
        role: Role,
    ) -> Result<UserRecord, AccountError> {
        require_manage_users(actor)?;
        require_account_role(role)?;
        let name = UserName::new(name)?;
        self.users.password_policy()?.check(password)?;
        let hash = self.hasher.hash(password)?;
        Ok(self.users.create(&name, role, &hash)?)
    }

    pub fn change_role(
        &self,
        actor: &Identity,
        id: UserId,
        role: Role,
    ) -> Result<(), AccountError> {
        require_manage_users(actor)?;
        require_account_role(role)?;
        if role != Role::Admin {
            self.keep_one_admin(id)?;
        }
        Ok(self.users.set_role(id, role)?)
    }

    pub fn set_active(
        &self,
        actor: &Identity,
        id: UserId,
        active: bool,
    ) -> Result<(), AccountError> {
        require_manage_users(actor)?;
        if !active {
            self.keep_one_admin(id)?;
        }
        self.users.set_active(id, active)?;
        if !active {
            self.sessions.delete_for_user(id)?;
        }
        Ok(())
    }

    /// Sets a new password; all sessions of the user end.
    pub fn set_password(
        &self,
        actor: &Identity,
        id: UserId,
        password: &str,
    ) -> Result<(), AccountError> {
        require_manage_users(actor)?;
        self.users.password_policy()?.check(password)?;
        let hash = self.hasher.hash(password)?;
        self.users.set_password_hash(id, &hash)?;
        Ok(self.sessions.delete_for_user(id)?)
    }

    /// An account changes its own password and proves it with the current one. All sessions of
    /// the user end, this one too. Guests and API tokens cannot do this. Wrong current passwords
    /// count as failed logins.
    pub fn change_own_password(
        &self,
        actor: &Identity,
        current: &str,
        new: &str,
    ) -> Result<(), AccountError> {
        let (Some((id, name)), None) = (&actor.user, actor.scope) else {
            return Err(AccountError::Forbidden);
        };
        let key = name.key();
        self.ensure_not_blocked(&key)?;
        let record = self.users.find(*id)?.ok_or(AccountError::NotFound)?;
        if !self.hasher.verify(current, &record.password_hash) {
            self.note_failure(&key);
            return Err(AccountError::WrongCurrentPassword);
        }
        self.users.password_policy()?.check(new)?;
        let hash = self.hasher.hash(new)?;
        self.users.set_password_hash(*id, &hash)?;
        self.clear_failures(&key);
        Ok(self.sessions.delete_for_user(*id)?)
    }

    pub fn delete_user(&self, actor: &Identity, id: UserId) -> Result<(), AccountError> {
        require_manage_users(actor)?;
        self.keep_one_admin(id)?;
        Ok(self.users.delete(id)?)
    }

    pub fn set_guest_enabled(&self, actor: &Identity, enabled: bool) -> Result<(), AccountError> {
        require_manage_users(actor)?;
        Ok(self.users.set_guest_enabled(enabled)?)
    }

    /// Sets the rules for new passwords; passwords set earlier stay valid.
    pub fn set_password_policy(
        &self,
        actor: &Identity,
        policy: PasswordPolicy,
    ) -> Result<(), AccountError> {
        require_manage_users(actor)?;
        Ok(self.users.set_password_policy(policy)?)
    }

    /// Fails if `id` is the last active admin, which must stay one.
    fn keep_one_admin(&self, id: UserId) -> Result<(), AccountError> {
        let users = self.users.list()?;
        let is_active_admin = |user: &UserRecord| user.role == Role::Admin && user.active;
        let target_is_admin = users
            .iter()
            .any(|user| user.id == id && is_active_admin(user));
        let others = users
            .iter()
            .filter(|user| user.id != id && is_active_admin(user))
            .count();
        if target_is_admin && others == 0 {
            return Err(AccountError::LastAdmin);
        }
        Ok(())
    }

    fn open_session(&self, record: &UserRecord) -> Result<Session, AccountError> {
        let token = self.tokens.issue();
        self.sessions.save(
            &self.tokens.digest(&token),
            record.id,
            self.clock.unix_seconds(),
        )?;
        Ok(Session {
            token,
            identity: Identity::of(record),
        })
    }

    fn ensure_not_blocked(&self, key: &str) -> Result<(), AccountError> {
        let now = self.clock.unix_seconds();
        let failures = self.failures.lock().unwrap_or_else(PoisonError::into_inner);
        match failures.get(key) {
            Some(entry) if entry.blocked_until > now => Err(AccountError::TooManyAttempts {
                retry_after_seconds: entry.blocked_until - now,
            }),
            _ => Ok(()),
        }
    }

    fn note_failure(&self, key: &str) {
        let now = self.clock.unix_seconds();
        let mut failures = self.failures.lock().unwrap_or_else(PoisonError::into_inner);
        let entry = failures.entry(key.to_owned()).or_insert(Failures {
            count: 0,
            blocked_until: 0,
        });
        entry.count = entry.count.saturating_add(1);
        if entry.count > FREE_ATTEMPTS {
            let doublings = (entry.count - FREE_ATTEMPTS - 1).min(6);
            let delay = (1u64 << doublings).min(MAX_DELAY_SECONDS);
            entry.blocked_until = now + delay;
        }
    }

    fn clear_failures(&self, key: &str) {
        self.failures
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(key);
    }
}

fn require_manage_users(actor: &Identity) -> Result<(), AccountError> {
    if actor.allows(Capability::ManageUsers) {
        Ok(())
    } else {
        Err(AccountError::Forbidden)
    }
}

fn require_account_role(role: Role) -> Result<(), AccountError> {
    if role == Role::Guest {
        Err(AccountError::GuestAccount)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{CountingTokens, FakeUsers, FakeWallClock, PrefixHasher};

    const PASSWORD: &str = "correct horse";

    struct Fixture {
        service: AccountService,
        clock: Arc<FakeWallClock>,
        users: Arc<FakeUsers>,
    }

    fn fixture() -> Fixture {
        let users = Arc::new(FakeUsers::default());
        let clock = Arc::new(FakeWallClock::default());
        let service = AccountService::new(
            users.clone(),
            users.clone(),
            Arc::new(PrefixHasher),
            Arc::new(CountingTokens::default()),
            clock.clone(),
        )
        .unwrap();
        Fixture {
            service,
            clock,
            users,
        }
    }

    fn admin_of(fixture: &Fixture) -> Identity {
        fixture.service.setup("root", PASSWORD).unwrap().identity
    }

    fn id_of(fixture: &Fixture, name: &str) -> UserId {
        fixture
            .users
            .find_by_name(&name.to_lowercase())
            .unwrap()
            .unwrap()
            .id
    }

    #[test]
    fn setup_works_once_and_signs_in() {
        let fixture = fixture();
        assert!(fixture.service.status().unwrap().setup_needed);
        let session = fixture.service.setup("root", PASSWORD).unwrap();
        assert_eq!(session.identity.role, Role::Admin);
        assert!(!fixture.service.status().unwrap().setup_needed);
        assert!(matches!(
            fixture.service.setup("other", PASSWORD),
            Err(AccountError::SetupDone)
        ));
        assert_eq!(
            fixture.service.authenticate(&session.token).unwrap(),
            Some(session.identity)
        );
    }

    #[test]
    fn setup_refuses_short_passwords() {
        assert!(matches!(
            fixture().service.setup("root", "short"),
            Err(AccountError::Invalid(InvalidUser::PasswordTooShort {
                min: 6
            }))
        ));
    }

    #[test]
    fn only_an_admin_sets_the_policy_and_it_applies_to_new_passwords() {
        let fixture = fixture();
        let admin = admin_of(&fixture);
        let ann = fixture
            .service
            .create_user(&admin, "ann", "simple", Role::User)
            .unwrap();
        assert!(matches!(
            fixture
                .service
                .set_password_policy(&Identity::of(&ann), PasswordPolicy::Strict),
            Err(AccountError::Forbidden)
        ));

        fixture
            .service
            .set_password_policy(&admin, PasswordPolicy::Strict)
            .unwrap();
        assert_eq!(
            fixture.service.status().unwrap().password_policy,
            PasswordPolicy::Strict
        );
        assert!(matches!(
            fixture
                .service
                .create_user(&admin, "bob", "simple", Role::User),
            Err(AccountError::Invalid(InvalidUser::PasswordTooShort {
                min: 10
            }))
        ));
        assert!(matches!(
            fixture.service.set_password(&admin, ann.id, "alllowercase"),
            Err(AccountError::Invalid(InvalidUser::PasswordNeedsUppercase))
        ));
        fixture
            .service
            .create_user(&admin, "bob", "Strong-Pass1", Role::User)
            .unwrap();
        // The earlier, simple password still works.
        assert!(fixture.service.login("ann", "simple").is_ok());
    }

    #[test]
    fn login_checks_the_password_and_ignores_name_case() {
        let fixture = fixture();
        admin_of(&fixture);
        assert!(fixture.service.login("ROOT", PASSWORD).is_ok());
        assert!(matches!(
            fixture.service.login("root", "wrong password"),
            Err(AccountError::BadCredentials)
        ));
        assert!(matches!(
            fixture.service.login("nobody", PASSWORD),
            Err(AccountError::BadCredentials)
        ));
    }

    #[test]
    fn failed_logins_are_delayed_with_growing_waits() {
        let fixture = fixture();
        admin_of(&fixture);
        for _ in 0..4 {
            let _ = fixture.service.login("root", "wrong password");
        }
        // Even the right password waits until the delay is over.
        assert!(matches!(
            fixture.service.login("root", PASSWORD),
            Err(AccountError::TooManyAttempts {
                retry_after_seconds: 1
            })
        ));
        fixture.clock.advance(1);
        let _ = fixture.service.login("root", "wrong password");
        assert!(matches!(
            fixture.service.login("root", PASSWORD),
            Err(AccountError::TooManyAttempts {
                retry_after_seconds: 2
            })
        ));
        fixture.clock.advance(2);
        assert!(fixture.service.login("root", PASSWORD).is_ok());
    }

    #[test]
    fn sessions_end_by_logout_and_by_idle_time() {
        let fixture = fixture();
        let session = fixture.service.setup("root", PASSWORD).unwrap();
        fixture.service.logout(&session.token).unwrap();
        assert_eq!(fixture.service.authenticate(&session.token).unwrap(), None);

        let session = fixture.service.login("root", PASSWORD).unwrap();
        fixture.clock.advance(SESSION_MAX_IDLE_SECONDS - 1);
        assert!(
            fixture
                .service
                .authenticate(&session.token)
                .unwrap()
                .is_some()
        );
        fixture.clock.advance(SESSION_MAX_IDLE_SECONDS + 1);
        assert_eq!(fixture.service.authenticate(&session.token).unwrap(), None);
    }

    #[test]
    fn guest_exists_only_while_enabled() {
        let fixture = fixture();
        let admin = admin_of(&fixture);
        assert!(matches!(
            fixture.service.guest(),
            Err(AccountError::GuestAccessOff)
        ));
        fixture.service.set_guest_enabled(&admin, true).unwrap();
        assert_eq!(fixture.service.guest().unwrap().role, Role::Guest);
        fixture.service.set_guest_enabled(&admin, false).unwrap();
        assert!(fixture.service.guest().is_err());
    }

    #[test]
    fn only_the_admin_manages_users() {
        let fixture = fixture();
        let admin = admin_of(&fixture);
        let user = fixture
            .service
            .create_user(&admin, "ann", PASSWORD, Role::User)
            .unwrap();
        let ann = Identity {
            user: Some((user.id, user.name)),
            role: Role::User,
            scope: None,
        };
        let guest = Identity::guest();
        for actor in [&ann, &guest] {
            assert!(matches!(
                fixture.service.list_users(actor),
                Err(AccountError::Forbidden)
            ));
            assert!(matches!(
                fixture
                    .service
                    .create_user(actor, "bob", PASSWORD, Role::Admin),
                Err(AccountError::Forbidden)
            ));
            assert!(matches!(
                fixture.service.change_role(actor, user.id, Role::Admin),
                Err(AccountError::Forbidden)
            ));
            assert!(matches!(
                fixture.service.set_active(actor, user.id, false),
                Err(AccountError::Forbidden)
            ));
            assert!(matches!(
                fixture.service.set_password(actor, user.id, PASSWORD),
                Err(AccountError::Forbidden)
            ));
            assert!(matches!(
                fixture.service.delete_user(actor, user.id),
                Err(AccountError::Forbidden)
            ));
            assert!(matches!(
                fixture.service.set_guest_enabled(actor, true),
                Err(AccountError::Forbidden)
            ));
        }
    }

    #[test]
    fn duplicate_names_and_guest_accounts_are_refused() {
        let fixture = fixture();
        let admin = admin_of(&fixture);
        assert!(matches!(
            fixture
                .service
                .create_user(&admin, "ROOT", PASSWORD, Role::User),
            Err(AccountError::NameTaken)
        ));
        assert!(matches!(
            fixture
                .service
                .create_user(&admin, "gus", PASSWORD, Role::Guest),
            Err(AccountError::GuestAccount)
        ));
    }

    #[test]
    fn the_last_active_admin_stays() {
        let fixture = fixture();
        let admin = admin_of(&fixture);
        let root = id_of(&fixture, "root");
        for result in [
            fixture.service.change_role(&admin, root, Role::User),
            fixture.service.set_active(&admin, root, false),
            fixture.service.delete_user(&admin, root),
        ] {
            assert!(matches!(result, Err(AccountError::LastAdmin)));
        }
        fixture
            .service
            .create_user(&admin, "second", PASSWORD, Role::Admin)
            .unwrap();
        fixture
            .service
            .change_role(&admin, root, Role::User)
            .unwrap();
    }

    #[test]
    fn an_account_changes_its_own_password_with_the_current_one() {
        let fixture = fixture();
        let admin = admin_of(&fixture);
        let user = fixture
            .service
            .create_user(&admin, "ann", PASSWORD, Role::User)
            .unwrap();
        let ann = Identity::of(&user);
        let session = fixture.service.login("ann", PASSWORD).unwrap();

        assert!(matches!(
            fixture
                .service
                .change_own_password(&ann, "wrong one", "another secret"),
            Err(AccountError::WrongCurrentPassword)
        ));
        assert!(matches!(
            fixture.service.change_own_password(&ann, PASSWORD, "short"),
            Err(AccountError::Invalid(_))
        ));
        assert!(
            fixture
                .service
                .authenticate(&session.token)
                .unwrap()
                .is_some()
        );

        fixture
            .service
            .change_own_password(&ann, PASSWORD, "another secret")
            .unwrap();
        assert_eq!(fixture.service.authenticate(&session.token).unwrap(), None);
        assert!(fixture.service.login("ann", PASSWORD).is_err());
        fixture.service.login("ann", "another secret").unwrap();
    }

    #[test]
    fn guests_and_api_tokens_cannot_change_a_password() {
        let fixture = fixture();
        let admin = admin_of(&fixture);
        let user = fixture
            .service
            .create_user(&admin, "ann", PASSWORD, Role::User)
            .unwrap();
        let by_token = Identity::of_token(&user, TokenScope::EditBookmarks);
        for actor in [Identity::guest(), by_token] {
            assert!(matches!(
                fixture
                    .service
                    .change_own_password(&actor, PASSWORD, "another secret"),
                Err(AccountError::Forbidden)
            ));
        }
    }

    #[test]
    fn deactivating_or_changing_the_password_ends_sessions() {
        let fixture = fixture();
        let admin = admin_of(&fixture);
        fixture
            .service
            .create_user(&admin, "ann", PASSWORD, Role::User)
            .unwrap();
        let ann = id_of(&fixture, "ann");

        let session = fixture.service.login("ann", PASSWORD).unwrap();
        fixture
            .service
            .set_password(&admin, ann, "another secret")
            .unwrap();
        assert_eq!(fixture.service.authenticate(&session.token).unwrap(), None);
        assert!(fixture.service.login("ann", PASSWORD).is_err());

        let session = fixture.service.login("ann", "another secret").unwrap();
        fixture.service.set_active(&admin, ann, false).unwrap();
        assert_eq!(fixture.service.authenticate(&session.token).unwrap(), None);
        assert!(matches!(
            fixture.service.login("ann", "another secret"),
            Err(AccountError::BadCredentials)
        ));
    }
}
