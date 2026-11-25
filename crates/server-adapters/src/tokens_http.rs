//! HTTP endpoints for the API tokens of the signed-in user.

use crate::{
    access::Caller,
    http::{ApiError, blocking},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
};
use std::sync::Arc;
use veltrunox_core::{ApiTokenId, Refusal};
use veltrunox_protocol::{
    ApiTokenDto, ApiTokenListDto, CreatedApiTokenDto, NewApiTokenDto, TOKEN_PATH, TOKENS_PATH,
    token_scope_from_dto, token_scope_to_dto,
};
use veltrunox_server_app::{ApiTokenRecord, ApiTokenService, TokenError};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(TOKENS_PATH, get(list_tokens).post(create_token))
        .route(TOKEN_PATH, delete(revoke_token))
}

fn token_to_dto(record: &ApiTokenRecord) -> ApiTokenDto {
    ApiTokenDto {
        id: record.id.value(),
        name: record.name.as_str().to_owned(),
        scope: token_scope_to_dto(record.scope),
        created_at: record.created_at,
        last_used: record.last_used,
    }
}

fn service(app: &AppState) -> Result<Arc<ApiTokenService>, ApiError> {
    app.api_tokens.clone().ok_or(ApiError::NotFound)
}

fn refused(error: &TokenError) -> ApiError {
    let status = match error {
        TokenError::Invalid(_) => StatusCode::BAD_REQUEST,
        TokenError::NotFound => StatusCode::NOT_FOUND,
        TokenError::Forbidden => StatusCode::FORBIDDEN,
        TokenError::Storage(_) => return ApiError::Internal,
    };
    ApiError::Refused(status, error.reason())
}

async fn list_tokens(
    State(app): State<AppState>,
    caller: Caller,
) -> Result<Json<ApiTokenListDto>, ApiError> {
    let tokens = service(&app)?;
    let records = blocking(move || tokens.list(&caller.0).map_err(|error| refused(&error))).await?;
    Ok(Json(ApiTokenListDto {
        tokens: records.iter().map(token_to_dto).collect(),
    }))
}

async fn create_token(
    State(app): State<AppState>,
    caller: Caller,
    Json(body): Json<NewApiTokenDto>,
) -> Result<(StatusCode, Json<CreatedApiTokenDto>), ApiError> {
    let tokens = service(&app)?;
    let created = blocking(move || {
        tokens
            .create(&caller.0, &body.name, token_scope_from_dto(body.scope))
            .map_err(|error| refused(&error))
    })
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedApiTokenDto {
            token: created.token,
            api_token: token_to_dto(&created.record),
        }),
    ))
}

async fn revoke_token(
    State(app): State<AppState>,
    caller: Caller,
    Path(id): Path<u64>,
) -> Result<StatusCode, ApiError> {
    let tokens = service(&app)?;
    blocking(move || {
        tokens
            .revoke(&caller.0, ApiTokenId::new(id))
            .map_err(|error| refused(&error))
    })
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
