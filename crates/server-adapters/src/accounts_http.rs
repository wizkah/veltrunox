//! HTTP endpoints for sign-in, rights and user administration.

use crate::{
    access::{AccessError, Caller, bearer_token},
    http::{ApiError, blocking, refusal},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use std::sync::Arc;
use veltrunox_core::{Refusal, UserId};
use veltrunox_protocol::{
    ChangePasswordDto, CredentialsDto, GUEST_ACCESS_PATH, GuestAccessDto, IdentityDto, LOGIN_PATH,
    LOGOUT_PATH, ME_PASSWORD_PATH, ME_PATH, NewUserDto, PASSWORD_POLICY_PATH, PROTOCOL_VERSION,
    SETUP_PATH, STATUS_PATH, SessionDto, SetPasswordPolicyDto, StatusDto, USER_PATH, USERS_PATH,
    UpdateUserDto, UserDto, capability_to_dto, password_policy_from_dto, password_policy_to_dto,
    role_from_dto, role_to_dto,
};
use veltrunox_server_app::{AccountError, AccountService, Identity, Session, UserRecord};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(STATUS_PATH, get(status))
        .route(SETUP_PATH, post(setup))
        .route(LOGIN_PATH, post(login))
        .route(LOGOUT_PATH, post(logout))
        .route(ME_PATH, get(me))
        .route(ME_PASSWORD_PATH, put(change_own_password))
        .route(USERS_PATH, get(list_users).post(create_user))
        .route(USER_PATH, put(update_user).delete(delete_user))
        .route(GUEST_ACCESS_PATH, put(set_guest_access))
        .route(PASSWORD_POLICY_PATH, put(set_password_policy))
}

pub fn identity_to_dto(identity: &Identity) -> IdentityDto {
    IdentityDto {
        user_id: identity.user.as_ref().map(|(id, _)| id.value()),
        user_name: identity
            .user
            .as_ref()
            .map(|(_, name)| name.as_str().to_owned()),
        role: role_to_dto(identity.role),
        capabilities: identity
            .capabilities()
            .into_iter()
            .map(capability_to_dto)
            .collect(),
    }
}

fn session_to_dto(session: &Session) -> SessionDto {
    SessionDto {
        token: session.token.clone(),
        identity: identity_to_dto(&session.identity),
    }
}

fn user_to_dto(record: &UserRecord) -> UserDto {
    UserDto {
        id: record.id.value(),
        user_name: record.name.as_str().to_owned(),
        role: role_to_dto(record.role),
        active: record.active,
    }
}

pub fn account_error_response(error: AccountError) -> Response {
    let reason = error.reason();
    match error {
        AccountError::Invalid(_) | AccountError::GuestAccount => {
            refusal(StatusCode::BAD_REQUEST, &reason)
        }
        AccountError::NameTaken | AccountError::SetupDone | AccountError::LastAdmin => {
            refusal(StatusCode::CONFLICT, &reason)
        }
        AccountError::NotFound => refusal(StatusCode::NOT_FOUND, &reason),
        AccountError::BadCredentials => refusal(StatusCode::UNAUTHORIZED, &reason),
        AccountError::WrongCurrentPassword => refusal(StatusCode::FORBIDDEN, &reason),
        AccountError::TooManyAttempts {
            retry_after_seconds,
        } => {
            let mut response = refusal(StatusCode::TOO_MANY_REQUESTS, &reason);
            if let Ok(value) = HeaderValue::from_str(&retry_after_seconds.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
            response
        }
        AccountError::Forbidden | AccountError::GuestAccessOff => {
            refusal(StatusCode::FORBIDDEN, &reason)
        }
        AccountError::Storage(_) | AccountError::Hash(_) => {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// The accounts of an app that requires sign-in; without them these endpoints do not exist.
fn accounts_of(app: &AppState) -> Result<Arc<AccountService>, ApiError> {
    app.accounts.clone().ok_or(ApiError::NotFound)
}

async fn status(State(app): State<AppState>) -> Result<Json<StatusDto>, ApiError> {
    let Some(accounts) = app.accounts.clone() else {
        return Ok(Json(StatusDto {
            protocol_version: PROTOCOL_VERSION,
            setup_needed: false,
            guest_enabled: true,
            password_policy: Default::default(),
        }));
    };
    let status = blocking(move || Ok(accounts.status()?)).await?;
    Ok(Json(StatusDto {
        protocol_version: PROTOCOL_VERSION,
        setup_needed: status.setup_needed,
        guest_enabled: status.guest_enabled,
        password_policy: password_policy_to_dto(status.password_policy),
    }))
}

async fn setup(
    State(app): State<AppState>,
    Json(body): Json<CredentialsDto>,
) -> Result<(StatusCode, Json<SessionDto>), ApiError> {
    let accounts = accounts_of(&app)?;
    let session = blocking(move || Ok(accounts.setup(&body.user_name, &body.password)?)).await?;
    Ok((StatusCode::CREATED, Json(session_to_dto(&session))))
}

async fn login(
    State(app): State<AppState>,
    Json(body): Json<CredentialsDto>,
) -> Result<Json<SessionDto>, ApiError> {
    let accounts = accounts_of(&app)?;
    let session = blocking(move || Ok(accounts.login(&body.user_name, &body.password)?)).await?;
    Ok(Json(session_to_dto(&session)))
}

async fn logout(State(app): State<AppState>, headers: HeaderMap) -> Result<StatusCode, ApiError> {
    let accounts = accounts_of(&app)?;
    let token = bearer_token(&headers)
        .ok_or(AccessError::Unauthorized)?
        .to_owned();
    blocking(move || Ok(accounts.logout(&token)?)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn me(caller: Caller) -> Json<IdentityDto> {
    Json(identity_to_dto(&caller.0))
}

/// The identity that performs an administration step. Without accounts nobody is managed.
fn actor(caller: Caller, app: &AppState) -> Result<(Identity, Arc<AccountService>), ApiError> {
    Ok((caller.0, accounts_of(app)?))
}

async fn list_users(
    State(app): State<AppState>,
    caller: Caller,
) -> Result<Json<Vec<UserDto>>, ApiError> {
    let (actor, accounts) = actor(caller, &app)?;
    let users = blocking(move || Ok(accounts.list_users(&actor)?)).await?;
    Ok(Json(users.iter().map(user_to_dto).collect()))
}

async fn create_user(
    State(app): State<AppState>,
    caller: Caller,
    Json(body): Json<NewUserDto>,
) -> Result<(StatusCode, Json<UserDto>), ApiError> {
    let (actor, accounts) = actor(caller, &app)?;
    let role = role_from_dto(body.role);
    let created = blocking(move || {
        Ok(accounts.create_user(&actor, &body.user_name, &body.password, role)?)
    })
    .await?;
    Ok((StatusCode::CREATED, Json(user_to_dto(&created))))
}

/// Applies the fields that are present, one after the other.
async fn update_user(
    State(app): State<AppState>,
    caller: Caller,
    Path(id): Path<u64>,
    Json(body): Json<UpdateUserDto>,
) -> Result<StatusCode, ApiError> {
    let (actor, accounts) = actor(caller, &app)?;
    let id = UserId::new(id);
    blocking(move || {
        if let Some(role) = body.role {
            accounts.change_role(&actor, id, role_from_dto(role))?;
        }
        if let Some(active) = body.active {
            accounts.set_active(&actor, id, active)?;
        }
        if let Some(password) = body.password {
            accounts.set_password(&actor, id, &password)?;
        }
        Ok(())
    })
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn change_own_password(
    State(app): State<AppState>,
    caller: Caller,
    Json(body): Json<ChangePasswordDto>,
) -> Result<StatusCode, ApiError> {
    let (actor, accounts) = actor(caller, &app)?;
    blocking(move || {
        Ok(accounts.change_own_password(&actor, &body.current_password, &body.new_password)?)
    })
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_user(
    State(app): State<AppState>,
    caller: Caller,
    Path(id): Path<u64>,
) -> Result<StatusCode, ApiError> {
    let (actor, accounts) = actor(caller, &app)?;
    blocking(move || Ok(accounts.delete_user(&actor, UserId::new(id))?)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_guest_access(
    State(app): State<AppState>,
    caller: Caller,
    Json(body): Json<GuestAccessDto>,
) -> Result<StatusCode, ApiError> {
    let (actor, accounts) = actor(caller, &app)?;
    blocking(move || Ok(accounts.set_guest_enabled(&actor, body.enabled)?)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_password_policy(
    State(app): State<AppState>,
    caller: Caller,
    Json(body): Json<SetPasswordPolicyDto>,
) -> Result<StatusCode, ApiError> {
    let (actor, accounts) = actor(caller, &app)?;
    let policy = password_policy_from_dto(body.policy);
    blocking(move || Ok(accounts.set_password_policy(&actor, policy)?)).await?;
    Ok(StatusCode::NO_CONTENT)
}
