use r2d2_sqlite;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, Error as SqliteError};
use std::path::Path;

pub type Pool = r2d2::Pool<SqliteConnectionManager>;
pub type PooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

const DB_FILENAME: &str = "database.sqlite";

pub fn get_pool() -> Pool {
    Pool::new(get_db_create_if_missing()).expect("Unable to create db pool.")
}

fn get_db_create_if_missing() -> SqliteConnectionManager {
    if !Path::new(DB_FILENAME).exists() {
        create_db()
    }
    SqliteConnectionManager::file(DB_FILENAME)
}

fn create_db() {
    let db = Connection::open(DB_FILENAME)
        .unwrap_or_else(|_| panic!(format!("Unable to open database file {}", DB_FILENAME)));

    db.execute_batch(include_str!("document/schema.sql"))
        .expect("Failed to execute document schema.");
    db.execute_batch(include_str!("user/schema.sql"))
        .expect("Failed to execute user schema.");

    enable_write_ahead_logging(&db);
}

fn enable_write_ahead_logging(db: &Connection) {
    // PRAGMA journal_mode=wal;
    let result: String = db
        .pragma_update_and_check(None, "journal_mode", &"wal", |row| row.get(0))
        .unwrap();
    assert!("wal" == &result);
}

pub fn is_primary_key_constraint(error: &SqliteError) -> bool {
    use libsqlite3_sys::ErrorCode::ConstraintViolation;
    use libsqlite3_sys::{Error as LibSqliteError, SQLITE_CONSTRAINT_PRIMARYKEY};
    use rusqlite::Error::SqliteFailure;

    match error {
        SqliteFailure(
            LibSqliteError {
                code: ConstraintViolation,
                extended_code: SQLITE_CONSTRAINT_PRIMARYKEY,
            },
            _,
        ) => true,
        _ => false,
    }
}
