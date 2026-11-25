//! SQLite storage of API tokens.

use crate::sqlite::{SqliteStore, corrupt, sql_int, storage, unsigned};
use rusqlite::{OptionalExtension, params};
use veltrunox_core::{ApiTokenId, TokenName, TokenScope, UserId};
use veltrunox_server_app::{
    ApiTokenRecord, ApiTokenRepository, StorageError, UserRecord, UserRepository,
};

fn scope_text(scope: TokenScope) -> &'static str {
    match scope {
        TokenScope::ReadOnly => "read_only",
        TokenScope::EditBookmarks => "edit_bookmarks",
    }
}

fn scope_from_text(text: &str) -> Result<TokenScope, StorageError> {
    match text {
        "read_only" => Ok(TokenScope::ReadOnly),
        "edit_bookmarks" => Ok(TokenScope::EditBookmarks),
        other => Err(corrupt("token scope", other)),
    }
}

impl ApiTokenRepository for SqliteStore {
    fn create(
        &self,
        user: UserId,
        name: &TokenName,
        scope: TokenScope,
        digest: &str,
        now: u64,
    ) -> Result<ApiTokenRecord, StorageError> {
        let connection = self.connection();
        connection
            .execute(
                "INSERT INTO api_tokens (user_id, name, digest, scope, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    sql_int(user.value())?,
                    name.as_str(),
                    digest,
                    scope_text(scope),
                    sql_int(now)?
                ],
            )
            .map_err(storage)?;
        Ok(ApiTokenRecord {
            id: ApiTokenId::new(unsigned(connection.last_insert_rowid())?),
            user,
            name: name.clone(),
            scope,
            created_at: now,
            last_used: None,
        })
    }

    fn list_for_user(&self, user: UserId) -> Result<Vec<ApiTokenRecord>, StorageError> {
        let connection = self.connection();
        let mut statement = connection
            .prepare(
                "SELECT id, name, scope, created_at, last_used FROM api_tokens \
                 WHERE user_id = ?1 ORDER BY id",
            )
            .map_err(storage)?;
        let rows = statement
            .query_map(params![sql_int(user.value())?], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                ))
            })
            .map_err(storage)?;
        rows.map(|row| {
            let (id, name, scope, created_at, last_used) = row.map_err(storage)?;
            Ok(ApiTokenRecord {
                id: ApiTokenId::new(unsigned(id)?),
                user,
                name: TokenName::new(&name).map_err(|error| corrupt("token name", error))?,
                scope: scope_from_text(&scope)?,
                created_at: unsigned(created_at)?,
                last_used: last_used.map(unsigned).transpose()?,
            })
        })
        .collect()
    }

    fn user_of(
        &self,
        digest: &str,
        now: u64,
    ) -> Result<Option<(UserRecord, TokenScope)>, StorageError> {
        let connection = self.connection();
        let found = connection
            .query_row(
                "SELECT t.id, t.scope, u.id FROM api_tokens t JOIN users u ON u.id = t.user_id \
                 WHERE t.digest = ?1 AND u.active = 1",
                params![digest],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(storage)?;
        let Some((token, scope, user)) = found else {
            return Ok(None);
        };
        connection
            .execute(
                "UPDATE api_tokens SET last_used = ?2 WHERE id = ?1",
                params![token, sql_int(now)?],
            )
            .map_err(storage)?;
        drop(connection);
        let scope = scope_from_text(&scope)?;
        let owner = UserRepository::find(self, UserId::new(unsigned(user)?))?;
        Ok(owner.map(|record| (record, scope)))
    }

    fn revoke(&self, user: UserId, id: ApiTokenId) -> Result<bool, StorageError> {
        let removed = self
            .connection()
            .execute(
                "DELETE FROM api_tokens WHERE id = ?1 AND user_id = ?2",
                params![sql_int(id.value())?, sql_int(user.value())?],
            )
            .map_err(storage)?;
        Ok(removed > 0)
    }
}
