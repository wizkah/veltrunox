//! Command line of the server: either run it, or administer users on this machine.

use std::path::PathBuf;
use veltrunox_core::{PasswordPolicy, Role};

pub const USAGE: &str = "\
usage:
  veltrunox-server [--config <file>]                  run the server
  veltrunox-server [--config <file>] users <command>  administer users without a running server

user commands:
  list                           show all users, the guest switch and the password rules
  add <name> [admin|user]        create a user (default role: user); asks for the password
  passwd <name>                  set a new password; ends the sessions of that user
  role <name> admin|user         change the role
  enable <name> | disable <name> allow or block the sign-in; disabling ends the sessions
  delete <name>                  delete a user
  guest on|off                   switch guest access
  policy standard|strict         set the rules for new passwords

add and passwd read the password from standard input instead of asking when given
--password-stdin. Run as the user that owns the data directory and from the directory the service runs in
(relative paths in the configuration), for example `sudo -u veltrunox sh -c 'cd /var/lib/veltrunox
&& veltrunox-server --config /etc/veltrunox/veltrunox.toml users list'`.";

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub config: Option<PathBuf>,
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Serve,
    Users(UserCommand),
}

/// Where a new password comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordSource {
    /// Asked on the terminal, twice, without echo.
    Prompt,
    /// One line of standard input, for scripts.
    Stdin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserCommand {
    List,
    Add {
        name: String,
        role: Role,
        password: PasswordSource,
    },
    SetPassword {
        name: String,
        password: PasswordSource,
    },
    SetRole {
        name: String,
        role: Role,
    },
    SetActive {
        name: String,
        active: bool,
    },
    Delete {
        name: String,
    },
    Guest {
        enabled: bool,
    },
    Policy(PasswordPolicy),
}

/// Parses the arguments after the program name; the error is the text to show.
pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Invocation, String> {
    let mut arguments = arguments.into_iter();
    let mut config = None;
    let mut password = PasswordSource::Prompt;
    let mut words = Vec::new();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--config" => {
                let path = arguments.next().ok_or(USAGE)?;
                if config.replace(PathBuf::from(path)).is_some() {
                    return Err(USAGE.to_owned());
                }
            }
            "--password-stdin" => password = PasswordSource::Stdin,
            _ => words.push(argument),
        }
    }
    let command = match words.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        [] if password == PasswordSource::Prompt => Command::Serve,
        ["users", ref rest @ ..] => Command::Users(parse_user_command(rest, password)?),
        _ => return Err(USAGE.to_owned()),
    };
    Ok(Invocation { config, command })
}

fn parse_user_command(words: &[&str], password: PasswordSource) -> Result<UserCommand, String> {
    let usage = || USAGE.to_owned();
    let command = match *words {
        ["list"] => UserCommand::List,
        ["add", name] => add(name, Role::User, password),
        ["add", name, role] => add(name, parse_role(role).ok_or_else(usage)?, password),
        ["passwd", name] => UserCommand::SetPassword {
            name: name.to_owned(),
            password,
        },
        ["role", name, role] => UserCommand::SetRole {
            name: name.to_owned(),
            role: parse_role(role).ok_or_else(usage)?,
        },
        ["enable", name] => set_active(name, true),
        ["disable", name] => set_active(name, false),
        ["delete", name] => UserCommand::Delete {
            name: name.to_owned(),
        },
        ["guest", "on"] => UserCommand::Guest { enabled: true },
        ["guest", "off"] => UserCommand::Guest { enabled: false },
        ["policy", "standard"] => UserCommand::Policy(PasswordPolicy::Standard),
        ["policy", "strict"] => UserCommand::Policy(PasswordPolicy::Strict),
        _ => return Err(usage()),
    };
    let takes_password = matches!(
        command,
        UserCommand::Add { .. } | UserCommand::SetPassword { .. }
    );
    if password == PasswordSource::Stdin && !takes_password {
        return Err(usage());
    }
    Ok(command)
}

fn add(name: &str, role: Role, password: PasswordSource) -> UserCommand {
    UserCommand::Add {
        name: name.to_owned(),
        role,
        password,
    }
}

fn set_active(name: &str, active: bool) -> UserCommand {
    UserCommand::SetActive {
        name: name.to_owned(),
        active,
    }
}

/// Accounts have the role admin or user; guests have no account.
fn parse_role(text: &str) -> Option<Role> {
    match text {
        "admin" => Some(Role::Admin),
        "user" => Some(Role::User),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_words(line: &str) -> Result<Invocation, String> {
        parse(line.split_whitespace().map(str::to_owned))
    }

    #[test]
    fn no_arguments_run_the_server() {
        let invocation = parse_words("").unwrap();
        assert_eq!(invocation.command, Command::Serve);
        assert_eq!(invocation.config, None);
    }

    #[test]
    fn config_may_come_before_or_after_the_command() {
        let before = parse_words("--config a.toml users list").unwrap();
        let after = parse_words("users list --config a.toml").unwrap();
        assert_eq!(before, after);
        assert_eq!(before.config, Some(PathBuf::from("a.toml")));
        assert_eq!(before.command, Command::Users(UserCommand::List));
    }

    #[test]
    fn add_defaults_to_the_user_role() {
        let Command::Users(UserCommand::Add { role, password, .. }) =
            parse_words("users add anna").unwrap().command
        else {
            panic!("not an add command");
        };
        assert_eq!(role, Role::User);
        assert_eq!(password, PasswordSource::Prompt);
        let Command::Users(UserCommand::Add { role, .. }) =
            parse_words("users add anna admin").unwrap().command
        else {
            panic!("not an add command");
        };
        assert_eq!(role, Role::Admin);
    }

    #[test]
    fn password_from_stdin_only_for_commands_that_take_one() {
        assert!(parse_words("users passwd anna --password-stdin").is_ok());
        assert!(parse_words("users add anna --password-stdin").is_ok());
        assert!(parse_words("users delete anna --password-stdin").is_err());
        assert!(parse_words("--password-stdin").is_err());
    }

    #[test]
    fn guests_are_not_an_account_role() {
        assert!(parse_words("users add anna guest").is_err());
        assert!(parse_words("users role anna guest").is_err());
    }

    #[test]
    fn unknown_or_incomplete_input_shows_the_usage() {
        for line in [
            "users",
            "users frobnicate",
            "users add",
            "users guest maybe",
            "--config",
            "serve",
            "--config a --config b",
        ] {
            assert_eq!(parse_words(line), Err(USAGE.to_owned()), "{line}");
        }
    }
}
