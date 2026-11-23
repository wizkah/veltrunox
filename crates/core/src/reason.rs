//! Stable identifiers for refusals, so interfaces can translate them.

use std::fmt::Display;

/// Why something was refused: a stable code, the numbers its text needs (a
/// limit, a number of seconds) and an English description for logs and for
/// codes an interface does not know. The wording shown to people belongs to
/// the interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason {
    pub code: String,
    pub args: Vec<u64>,
    pub message: String,
}

impl Reason {
    /// A reason made from a code and an English description.
    pub fn coded(code: &str, message: &str) -> Self {
        Self::new(&message, code)
    }

    pub fn new(error: &impl Display, code: &str) -> Self {
        Self::with_args(error, code, Vec::new())
    }

    pub fn with_args(error: &impl Display, code: &str, args: Vec<u64>) -> Self {
        Self {
            code: code.to_owned(),
            args,
            message: error.to_string(),
        }
    }
}

/// An error that can say why in terms of a [`Reason`].
pub trait Refusal {
    fn reason(&self) -> Reason;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InvalidUser, InvalidValue, TransportError};

    #[test]
    fn a_refusal_names_a_code_and_keeps_its_numbers() {
        let reason = InvalidUser::PasswordTooShort { min: 10 }.reason();
        assert_eq!(reason.code, "password_too_short");
        assert_eq!(reason.args, vec![10]);
        assert_eq!(reason.message, "password must have at least 10 characters");
        assert_eq!(
            InvalidValue::SpeedOutOfRange.reason().code,
            "speed_out_of_range"
        );
        assert!(TransportError::NoRecording.reason().args.is_empty());
    }
}
