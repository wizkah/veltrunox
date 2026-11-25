//! User administration on the machine that holds the data, without a running server.
//!
//! Whoever may run this has the data directory in reach, so it acts as an admin; the rules of
//! the account service (names, password rules, the last admin) apply as everywhere.

use crate::cli::{PasswordSource, UserCommand};
use std::io::{self, BufRead, Write};
use veltrunox_core::{PasswordPolicy, Role};
use veltrunox_server_app::{AccountError, AccountService, Identity, UserRecord};

#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error("no user named \"{0}\"")]
    UnknownUser(String),
    #[error("the two passwords differ")]
    PasswordsDiffer,
    #[error("cannot read the password: {0}")]
    Password(io::Error),
    #[error("cannot write the output: {0}")]
    Output(io::Error),
}

/// Where passwords come from; the terminal in production, a fixed answer in tests.
pub trait PasswordInput {
    fn read(&mut self, source: PasswordSource) -> Result<String, AdminError>;
}

/// Executes one command and reports what it did.
pub fn run(
    accounts: &AccountService,
    command: &UserCommand,
    passwords: &mut dyn PasswordInput,
    output: &mut dyn Write,
) -> Result<(), AdminError> {
    let admin = Identity {
        user: None,
        role: Role::Admin,
        scope: None,
    };
    let message = match command {
        UserCommand::List => return list(accounts, &admin, output),
        UserCommand::Add {
            name,
            role,
            password,
        } => {
            let password = passwords.read(*password)?;
            let user = accounts.create_user(&admin, name, &password, *role)?;
            format!("created {} ({})", user.name.as_str(), role_name(user.role))
        }
        UserCommand::SetPassword { name, password } => {
            let user = find(accounts, &admin, name)?;
            let password = passwords.read(*password)?;
            accounts.set_password(&admin, user.id, &password)?;
            format!(
                "new password for {}; their sessions ended",
                user.name.as_str()
            )
        }
        UserCommand::SetRole { name, role } => {
            let user = find(accounts, &admin, name)?;
            accounts.change_role(&admin, user.id, *role)?;
            format!("{} is now {}", user.name.as_str(), role_name(*role))
        }
        UserCommand::SetActive { name, active } => {
            let user = find(accounts, &admin, name)?;
            accounts.set_active(&admin, user.id, *active)?;
            let state = if *active { "enabled" } else { "disabled" };
            format!("{} {state}", user.name.as_str())
        }
        UserCommand::Delete { name } => {
            let user = find(accounts, &admin, name)?;
            accounts.delete_user(&admin, user.id)?;
            format!("deleted {}", user.name.as_str())
        }
        UserCommand::Guest { enabled } => {
            accounts.set_guest_enabled(&admin, *enabled)?;
            format!("guest access {}", on_off(*enabled))
        }
        UserCommand::Policy(policy) => {
            accounts.set_password_policy(&admin, *policy)?;
            format!("password rules: {}", policy_name(*policy))
        }
    };
    writeln!(output, "{message}").map_err(AdminError::Output)
}

fn list(
    accounts: &AccountService,
    admin: &Identity,
    output: &mut dyn Write,
) -> Result<(), AdminError> {
    let users = accounts.list_users(admin)?;
    let status = accounts.status()?;
    let write = |output: &mut dyn Write| -> io::Result<()> {
        if users.is_empty() {
            writeln!(output, "no users yet")?;
        }
        for user in &users {
            let blocked = if user.active { "" } else { "  (disabled)" };
            writeln!(
                output,
                "{:<6}{}{blocked}",
                role_name(user.role),
                user.name.as_str()
            )?;
        }
        writeln!(output, "guest access: {}", on_off(status.guest_enabled))?;
        writeln!(
            output,
            "password rules: {}",
            policy_name(status.password_policy)
        )
    };
    write(output).map_err(AdminError::Output)
}

fn find(accounts: &AccountService, admin: &Identity, name: &str) -> Result<UserRecord, AdminError> {
    let key = name.trim().to_lowercase();
    accounts
        .list_users(admin)?
        .into_iter()
        .find(|user| user.name.key() == key)
        .ok_or_else(|| AdminError::UnknownUser(name.to_owned()))
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Admin => "admin",
        Role::User => "user",
        Role::Guest => "guest",
    }
}

fn policy_name(policy: PasswordPolicy) -> &'static str {
    match policy {
        PasswordPolicy::Standard => "standard (at least 6 characters)",
        PasswordPolicy::Strict => {
            "strict (at least 10, with lower and upper case, digit, special character)"
        }
    }
}

fn on_off(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}

/// Reads passwords from the terminal (hidden, asked twice) or from standard input.
pub struct TerminalPasswords;

impl PasswordInput for TerminalPasswords {
    fn read(&mut self, source: PasswordSource) -> Result<String, AdminError> {
        match source {
            PasswordSource::Stdin => {
                let mut line = String::new();
                io::stdin()
                    .lock()
                    .read_line(&mut line)
                    .map_err(AdminError::Password)?;
                Ok(line.trim_end_matches(['\r', '\n']).to_owned())
            }
            PasswordSource::Prompt => {
                let first =
                    rpassword::prompt_password("Password: ").map_err(AdminError::Password)?;
                let second =
                    rpassword::prompt_password("Repeat: ").map_err(AdminError::Password)?;
                if first == second {
                    Ok(first)
                } else {
                    Err(AdminError::PasswordsDiffer)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use veltrunox_server_adapters::{Argon2Hasher, RandomTokens, SqliteStore, SystemWallClock};

    struct Fixed(&'static str);

    impl PasswordInput for Fixed {
        fn read(&mut self, _source: PasswordSource) -> Result<String, AdminError> {
            Ok(self.0.to_owned())
        }
    }

    fn accounts() -> (tempfile::TempDir, AccountService) {
        let directory = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(&directory.path().join("test.db")).unwrap());
        let service = AccountService::new(
            store.clone(),
            store,
            Arc::new(Argon2Hasher::new()),
            Arc::new(RandomTokens),
            Arc::new(SystemWallClock),
        )
        .unwrap();
        (directory, service)
    }

    fn execute(
        accounts: &AccountService,
        command: UserCommand,
        password: &'static str,
    ) -> Result<String, AdminError> {
        let mut output = Vec::new();
        run(accounts, &command, &mut Fixed(password), &mut output)?;
        Ok(String::from_utf8(output).unwrap())
    }

    fn add(name: &str, role: Role) -> UserCommand {
        UserCommand::Add {
            name: name.to_owned(),
            role,
            password: PasswordSource::Prompt,
        }
    }

    #[test]
    fn first_admin_can_be_created_and_signs_in() {
        let (_directory, accounts) = accounts();
        execute(&accounts, add("Anna", Role::Admin), "secret1").unwrap();
        assert!(!accounts.status().unwrap().setup_needed);
        let session = accounts.login("anna", "secret1").unwrap();
        assert_eq!(session.identity.role, Role::Admin);
    }

    #[test]
    fn new_password_replaces_the_old_one() {
        let (_directory, accounts) = accounts();
        execute(&accounts, add("anna", Role::Admin), "secret1").unwrap();
        let command = UserCommand::SetPassword {
            name: "ANNA".to_owned(),
            password: PasswordSource::Prompt,
        };
        execute(&accounts, command, "another2").unwrap();
        assert!(accounts.login("anna", "secret1").is_err());
        assert!(accounts.login("anna", "another2").is_ok());
    }

    #[test]
    fn the_last_admin_stays() {
        let (_directory, accounts) = accounts();
        execute(&accounts, add("anna", Role::Admin), "secret1").unwrap();
        let delete = UserCommand::Delete {
            name: "anna".to_owned(),
        };
        assert!(matches!(
            execute(&accounts, delete, ""),
            Err(AdminError::Account(AccountError::LastAdmin))
        ));
    }

    #[test]
    fn password_rules_apply() {
        let (_directory, accounts) = accounts();
        assert!(matches!(
            execute(&accounts, add("anna", Role::Admin), "abc"),
            Err(AdminError::Account(AccountError::Invalid(_)))
        ));
    }

    #[test]
    fn unknown_user_is_named() {
        let (_directory, accounts) = accounts();
        let command = UserCommand::SetActive {
            name: "nobody".to_owned(),
            active: false,
        };
        assert!(matches!(
            execute(&accounts, command, ""),
            Err(AdminError::UnknownUser(name)) if name == "nobody"
        ));
    }

    #[test]
    fn disabled_user_cannot_sign_in_and_listing_shows_the_state() {
        let (_directory, accounts) = accounts();
        execute(&accounts, add("anna", Role::Admin), "secret1").unwrap();
        execute(&accounts, add("ben", Role::User), "secret2").unwrap();
        let disable = UserCommand::SetActive {
            name: "ben".to_owned(),
            active: false,
        };
        execute(&accounts, disable, "").unwrap();
        assert!(accounts.login("ben", "secret2").is_err());
        let listing = execute(&accounts, UserCommand::List, "").unwrap();
        assert!(listing.contains("admin an"), "{listing}");
        assert!(listing.contains("user  ben  (disabled)"), "{listing}");
        assert!(listing.contains("guest access: off"), "{listing}");
    }

    #[test]
    fn guest_switch_and_password_rules_are_stored() {
        let (_directory, accounts) = accounts();
        execute(&accounts, UserCommand::Guest { enabled: true }, "").unwrap();
        execute(&accounts, UserCommand::Policy(PasswordPolicy::Strict), "").unwrap();
        let status = accounts.status().unwrap();
        assert!(status.guest_enabled);
        assert_eq!(status.password_policy, PasswordPolicy::Strict);
    }
}
