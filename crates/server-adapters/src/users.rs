//! SQLite storage of users, sessions and the guest switch.

use crate::sqlite::{SqliteStore, corrupt, sql_int, storage, unsigned};
use rusqlite::{ErrorCode, OptionalExtension, params};
use veltrunox_core::{Role, UserId, UserName};
use veltrunox_server_app::{
    SessionRepository, StorageError, UserRecord, UserRepository, UserStoreError,
};

const GUEST_KEY: &str = "guest_enabled";
const USER_COLUMNS: &str = "id, name, role, password_hash, active";

fn role_text(role: Role) -> &'static str {
    match role {
        Role::Admin => "admin",
        Role::User | Role::Guest => "user",
    }
}

fn user_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<(i64, String, String, String, i64)> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

fn record(
    (id, name, role, hash, active): (i64, String, String, String, i64),
) -> Result<UserRecord, StorageError> {
    Ok(UserRecord {
        id: UserId::new(unsigned(id)?),
        name: UserName::new(&name).map_err(|error| corrupt("user name", error))?,
        role: match role.as_str() {
            "admin" => Role::Admin,
            "user" => Role::User,
            other => return Err(corrupt("role", other)),
        },
        password_hash: hash,
        active: active != 0,
    })
}

fn is_name_clash(error: &rusqlite::Error) -> bool {
    matches!(error, rusqlite::Error::SqliteFailure(failure, _)
        if failure.code == ErrorCode::ConstraintViolation)
}

impl UserRepository for SqliteStore {
    fn count(&self) -> Result<u64, StorageError> {
        let count: i64 = self
            .connection()
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .map_err(storage)?;
        unsigned(count)
    }

    fn create_first_admin(
        &self,
        name: &UserName,
        password_hash: &str,
    ) -> Result<Option<UserRecord>, UserStoreError> {
        let connection = self.connection();
        // One statement: the check and the insert cannot be separated by another request.
        let added = connection
            .execute(
                "INSERT INTO users (name, name_key, role, password_hash)
                 SELECT ?1, ?2, 'admin', ?3 WHERE NOT EXISTS (SELECT 1 FROM users)",
                params![name.as_str(), name.key(), password_hash],
            )
            .map_err(storage)?;
        if added == 0 {
            return Ok(None);
        }
        Ok(Some(UserRecord {
            id: UserId::new(unsigned(connection.last_insert_rowid())?),
            name: name.clone(),
            role: Role::Admin,
            password_hash: password_hash.to_owned(),
            active: true,
        }))
    }

    fn create(
        &self,
        name: &UserName,
        role: Role,
        password_hash: &str,
    ) -> Result<UserRecord, UserStoreError> {
        let connection = self.connection();
        connection
            .execute(
                "INSERT INTO users (name, name_key, role, password_hash) VALUES (?1, ?2, ?3, ?4)",
                params![name.as_str(), name.key(), role_text(role), password_hash],
            )
            .map_err(|error| {
                if is_name_clash(&error) {
                    UserStoreError::NameTaken
                } else {
                    storage(error).into()
                }
            })?;
        Ok(UserRecord {
            id: UserId::new(unsigned(connection.last_insert_rowid())?),
            name: name.clone(),
            role: if role == Role::Admin {
                Role::Admin
            } else {
                Role::User
            },
            password_hash: password_hash.to_owned(),
            active: true,
        })
    }

    fn find_by_name(&self, key: &str) -> Result<Option<UserRecord>, StorageError> {
        self.connection()
            .query_row(
                &format!("SELECT {USER_COLUMNS} FROM users WHERE name_key = ?1"),
                params![key],
                user_from_row,
            )
            .optional()
            .map_err(storage)?
            .map(record)
            .transpose()
    }

    fn find(&self, id: UserId) -> Result<Option<UserRecord>, StorageError> {
        self.connection()
            .query_row(
                &format!("SELECT {USER_COLUMNS} FROM users WHERE id = ?1"),
                params![sql_int(id.value())?],
                user_from_row,
            )
            .optional()
            .map_err(storage)?
            .map(record)
            .transpose()
    }

    fn list(&self) -> Result<Vec<UserRecord>, StorageError> {
        let connection = self.connection();
        let mut statement = connection
            .prepare(&format!("SELECT {USER_COLUMNS} FROM users ORDER BY id"))
            .map_err(storage)?;
        let rows = statement.query_map([], user_from_row).map_err(storage)?;
        rows.map(|row| record(row.map_err(storage)?)).collect()
    }

    fn set_role(&self, id: UserId, role: Role) -> Result<(), UserStoreError> {
        self.update_user("role = ?2", id, role_text(role))
    }

    fn set_active(&self, id: UserId, active: bool) -> Result<(), UserStoreError> {
        self.update_user("active = ?2", id, i64::from(active))
    }

    fn set_password_hash(&self, id: UserId, password_hash: &str) -> Result<(), UserStoreError> {
        self.update_user("password_hash = ?2", id, password_hash)
    }

    fn delete(&self, id: UserId) -> Result<(), UserStoreError> {
        let removed = self
            .connection()
            .execute(
                "DELETE FROM users WHERE id = ?1",
                params![sql_int(id.value())?],
            )
            .map_err(storage)?;
        if removed == 0 {
            return Err(UserStoreError::NotFound);
        }
        Ok(())
    }

    fn guest_enabled(&self) -> Result<bool, StorageError> {
        let value = self
            .connection()
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![GUEST_KEY],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(storage)?;
        Ok(value.as_deref() == Some("1"))
    }

    fn set_guest_enabled(&self, enabled: bool) -> Result<(), StorageError> {
        self.connection()
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![GUEST_KEY, if enabled { "1" } else { "0" }],
            )
            .map_err(storage)?;
        Ok(())
    }
}

impl SqliteStore {
    fn update_user(
        &self,
        assignment: &str,
        id: UserId,
        value: impl rusqlite::ToSql,
    ) -> Result<(), UserStoreError> {
        let changed = self
            .connection()
            .execute(
                &format!("UPDATE users SET {assignment} WHERE id = ?1"),
                params![sql_int(id.value())?, value],
            )
            .map_err(storage)?;
        if changed == 0 {
            return Err(UserStoreError::NotFound);
        }
        Ok(())
    }
}

impl SessionRepository for SqliteStore {
    fn save(&self, digest: &str, user: UserId, now: u64) -> Result<(), StorageError> {
        let now = sql_int(now)?;
        self.connection()
            .execute(
                "INSERT INTO sessions (digest, user_id, created_at, last_used) VALUES (?1, ?2, ?3, ?3)",
                params![digest, sql_int(user.value())?, now],
            )
            .map_err(storage)?;
        Ok(())
    }

    fn user_of(
        &self,
        digest: &str,
        now: u64,
        max_idle: u64,
    ) -> Result<Option<UserRecord>, StorageError> {
        let now = sql_int(now)?;
        let oldest = now.saturating_sub(sql_int(max_idle)?);
        let connection = self.connection();
        connection
            .execute("DELETE FROM sessions WHERE last_used < ?1", params![oldest])
            .map_err(storage)?;
        let found = connection
            .query_row(
                "SELECT u.id, u.name, u.role, u.password_hash, u.active
                 FROM sessions s JOIN users u ON u.id = s.user_id
                 WHERE s.digest = ?1 AND u.active = 1",
                params![digest],
                user_from_row,
            )
            .optional()
            .map_err(storage)?;
        if found.is_some() {
            connection
                .execute(
                    "UPDATE sessions SET last_used = ?2 WHERE digest = ?1",
                    params![digest, now],
                )
                .map_err(storage)?;
        }
        found.map(record).transpose()
    }

    fn end(&self, digest: &str) -> Result<(), StorageError> {
        self.connection()
            .execute("DELETE FROM sessions WHERE digest = ?1", params![digest])
            .map_err(storage)?;
        Ok(())
    }

    fn delete_for_user(&self, user: UserId) -> Result<(), StorageError> {
        self.connection()
            .execute(
                "DELETE FROM sessions WHERE user_id = ?1",
                params![sql_int(user.value())?],
            )
            .map_err(storage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(text: &str) -> UserName {
        UserName::new(text).unwrap()
    }

    #[test]
    fn the_first_admin_is_created_only_once() {
        let store = SqliteStore::in_memory().unwrap();
        let first = store
            .create_first_admin(&name("root"), "h")
            .unwrap()
            .unwrap();
        assert_eq!(first.role, Role::Admin);
        assert_eq!(store.create_first_admin(&name("other"), "h").unwrap(), None);
        assert_eq!(store.count().unwrap(), 1);
    }

    #[test]
    fn names_are_unique_without_regard_to_case() {
        let store = SqliteStore::in_memory().unwrap();
        store.create(&name("Ann"), Role::User, "h").unwrap();
        assert!(matches!(
            store.create(&name("ANN"), Role::User, "h"),
            Err(UserStoreError::NameTaken)
        ));
        assert_eq!(
            store.find_by_name("ann").unwrap().unwrap().name.as_str(),
            "Ann"
        );
    }

    #[test]
    fn users_change_and_are_deleted_with_their_sessions() {
        let store = SqliteStore::in_memory().unwrap();
        let user = store.create(&name("ann"), Role::User, "h").unwrap();
        store.set_role(user.id, Role::Admin).unwrap();
        store.set_password_hash(user.id, "new").unwrap();
        store.set_active(user.id, false).unwrap();
        let stored = store.find(user.id).unwrap().unwrap();
        assert_eq!(
            (stored.role, stored.password_hash.as_str(), stored.active),
            (Role::Admin, "new", false)
        );
        store.set_active(user.id, true).unwrap();
        store.save("digest", user.id, 100).unwrap();
        store.delete(user.id).unwrap();
        assert_eq!(store.user_of("digest", 100, 1000).unwrap(), None);
        assert!(matches!(
            store.delete(user.id),
            Err(UserStoreError::NotFound)
        ));
        assert!(matches!(
            store.set_role(user.id, Role::User),
            Err(UserStoreError::NotFound)
        ));
    }

    #[test]
    fn sessions_expire_by_idle_time_and_use_renews_them() {
        let store = SqliteStore::in_memory().unwrap();
        let user = store.create(&name("ann"), Role::User, "h").unwrap();
        store.save("digest", user.id, 100).unwrap();
        assert!(store.user_of("digest", 150, 100).unwrap().is_some());
        // Used at 150, so 240 is within 100 seconds of use.
        assert!(store.user_of("digest", 240, 100).unwrap().is_some());
        assert_eq!(store.user_of("digest", 400, 100).unwrap(), None);
        assert_eq!(store.user_of("digest", 401, 100).unwrap(), None);
    }

    #[test]
    fn inactive_users_have_no_session() {
        let store = SqliteStore::in_memory().unwrap();
        let user = store.create(&name("ann"), Role::User, "h").unwrap();
        store.save("digest", user.id, 100).unwrap();
        store.set_active(user.id, false).unwrap();
        assert_eq!(store.user_of("digest", 100, 1000).unwrap(), None);
    }

    #[test]
    fn the_guest_switch_is_off_by_default_and_stored() {
        let store = SqliteStore::in_memory().unwrap();
        assert!(!store.guest_enabled().unwrap());
        store.set_guest_enabled(true).unwrap();
        assert!(store.guest_enabled().unwrap());
        store.set_guest_enabled(false).unwrap();
        assert!(!store.guest_enabled().unwrap());
    }
}
