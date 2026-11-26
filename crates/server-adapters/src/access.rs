//! Who is calling: the bearer token of a request, and the rights check.

use crate::http::refusal;
use crate::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use veltrunox_core::{Capability, Reason, Role};
use veltrunox_server_app::{AccountService, ApiTokenService, Identity};

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
        scope: None,
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

/// The identity behind a token of either kind: an API token, recognised by its prefix, or the
/// token of a login. `None` if it is unknown, expired or of a deactivated user. Blocking.
pub(crate) fn identify(
    accounts: &AccountService,
    api_tokens: Option<&ApiTokenService>,
    token: &str,
) -> Result<Option<Identity>, ()> {
    if ApiTokenService::is_api_token(token) {
        match api_tokens {
            Some(service) => service.authenticate(token).map_err(|_| ()),
            None => Ok(None),
        }
    } else {
        accounts.authenticate(token).map_err(|_| ())
    }
}

/// Whether the token is the owner token of this endpoint.
pub(crate) fn is_owner(owner_token: Option<&str>, token: &str) -> bool {
    owner_token.is_some_and(|owner| owner == token)
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
        let owner = app.owner_token.as_deref();
        if token.as_deref().is_some_and(|token| is_owner(owner, token)) {
            return Ok(Self(open_identity()));
        }
        let api_tokens = app.api_tokens.clone();
        // Looking up a session reads and writes the database, which blocks.
        let found = tokio::task::spawn_blocking(move || match token {
            Some(token) => identify(&accounts, api_tokens.as_deref(), &token),
            None => Ok(accounts.guest().ok()),
        })
        .await
        .map_err(|_| AccessError::Failed)?
        .map_err(|()| AccessError::Failed)?;
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
    fn only_the_exact_token_is_the_owner_and_only_where_one_is_set() {
        assert!(is_owner(Some("secret"), "secret"));
        assert!(!is_owner(Some("secret"), "other"));
        assert!(!is_owner(None, "secret"));
    }

    #[test]
    fn other_schemes_and_empty_tokens_are_ignored() {
        assert_eq!(bearer_token(&headers("Basic abc")), None);
        assert_eq!(bearer_token(&headers("Bearer ")), None);
        assert_eq!(bearer_token(&HeaderMap::new()), None);
    }
}
