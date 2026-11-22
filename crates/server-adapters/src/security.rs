//! Password hashing, session tokens and wall-clock time.

use argon2::{
    Argon2,
    password_hash::{PasswordHasher as _, PasswordVerifier as _, phc::PasswordHash},
};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};
use veltrunox_server_app::{HashFailure, PasswordHasher, SessionTokens, WallClock};

/// Argon2id with the library's recommended parameters; the salt is random per hash.
#[derive(Default)]
pub struct Argon2Hasher {
    argon2: Argon2<'static>,
}

impl Argon2Hasher {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PasswordHasher for Argon2Hasher {
    fn hash(&self, password: &str) -> Result<String, HashFailure> {
        self.argon2
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|error| HashFailure(error.to_string()))
    }

    fn verify(&self, password: &str, hash: &str) -> bool {
        PasswordHash::new(hash)
            .map(|parsed| {
                self.argon2
                    .verify_password(password.as_bytes(), &parsed)
                    .is_ok()
            })
            .unwrap_or(false)
    }
}

/// Tokens of 32 random bytes from the operating system, written as hex.
pub struct RandomTokens;

impl SessionTokens for RandomTokens {
    fn issue(&self) -> String {
        let mut bytes = [0u8; 32];
        // Without randomness no safe token exists; stopping is the only safe answer.
        #[allow(clippy::expect_used)]
        getrandom::fill(&mut bytes).expect("the operating system provides random bytes");
        hex(&bytes)
    }

    fn digest(&self, token: &str) -> String {
        hex(&Sha256::digest(token.as_bytes()))
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

pub struct SystemWallClock;

impl WallClock for SystemWallClock {
    fn unix_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_verify_only_the_right_password() {
        let hasher = Argon2Hasher::new();
        let hash = hasher.hash("correct horse").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(hasher.verify("correct horse", &hash));
        assert!(!hasher.verify("wrong horse", &hash));
        assert!(!hasher.verify("correct horse", "not a hash"));
    }

    #[test]
    fn equal_passwords_get_different_hashes() {
        let hasher = Argon2Hasher::new();
        assert_ne!(
            hasher.hash("correct horse").unwrap(),
            hasher.hash("correct horse").unwrap()
        );
    }

    #[test]
    fn tokens_differ_and_the_digest_is_stable() {
        let tokens = RandomTokens;
        let first = tokens.issue();
        assert_eq!(first.len(), 64);
        assert_ne!(first, tokens.issue());
        assert_eq!(tokens.digest(&first), tokens.digest(&first));
        assert_ne!(tokens.digest(&first), first);
    }
}
