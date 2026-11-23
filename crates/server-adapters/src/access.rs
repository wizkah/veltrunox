//! Who is calling: the bearer token of a request, and the rights check.

use crate::http::refusal;
use crate::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use veltrunox_core::{Capability, Reason, Role};
use veltrunox_server_app::{AccountError, Identity};

/// The caller of a request. Without accounts the server is open and everybody is an admin.
pub struct Caller(pub Identity);

impl Caller {
    /// 403 unless the caller holds the capability.
    pub fn require(&self, capability: Capability) -> Result<(), AccessError> {
        if self.0.allows(capability) {
            Ok(())
        } else {
            Err(AccessError::Forbidden)
        }
    }
}

pub fn open_identity() -> Identity {
    Identity {
        user: None,
        role: Role::Admin,
    }
}

pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    scheme
        .eq_ignore_ascii_case("bearer")
        .then_some(token.trim())
        .filter(|token| !token.is_empty())
}

/// Why a request is refused before any work is done.
#[derive(Debug)]
pub enum AccessError {
    /// No valid token.
    Unauthorized,
    /// Signed in, but the role lacks the capability.
    Forbidden,
    Failed,
}

impl IntoResponse for AccessError {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized => refusal(
                StatusCode::UNAUTHORIZED,
                &Reason::coded("sign_in_required", "sign-in required"),
            ),
            Self::Forbidden => refusal(
                StatusCode::FORBIDDEN,
                &Reason::coded("not_allowed", "not allowed"),
            ),
            Self::Failed => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl FromRequestParts<AppState> for Caller {
    type Rejection = AccessError;

    async fn from_request_parts(
        parts: &mut Parts,
        app: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(accounts) = app.accounts.clone() else {
            return Ok(Self(open_identity()));
        };
        // No token means a guest, while guests are let in; a wrong token never does.
        let token = bearer_token(&parts.headers).map(str::to_owned);
        // Looking up a session reads and writes the database, which blocks.
        let found = tokio::task::spawn_blocking(move || match token {
            Some(token) => accounts.authenticate(&token),
            None => Ok(accounts.guest().ok()),
        })
        .await
        .map_err(|_| AccessError::Failed)?
        .map_err(|_: AccountError| AccessError::Failed)?;
        found.map(Self).ok_or(AccessError::Unauthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_str(value).unwrap());
        headers
    }

    #[test]
    fn the_bearer_token_is_read_from_the_header() {
        assert_eq!(bearer_token(&headers("Bearer abc")), Some("abc"));
        assert_eq!(bearer_token(&headers("bearer abc")), Some("abc"));
    }

    #[test]
    fn other_schemes_and_empty_tokens_are_ignored() {
        assert_eq!(bearer_token(&headers("Basic abc")), None);
        assert_eq!(bearer_token(&headers("Bearer ")), None);
        assert_eq!(bearer_token(&HeaderMap::new()), None);
    }
}
