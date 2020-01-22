use r2d2_sqlite;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, Error as SqliteError};
use std::path::Path;

pub type Pool = r2d2::Pool<SqliteConnectionManager>;
pub type PooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

pub fn get_pool(filename: &str) -> Pool {
    Pool::new(get_db_create_if_missing(filename)).expect("Unable to create db pool.")
}

fn get_db_create_if_missing(filename: &str) -> SqliteConnectionManager {
    if !Path::new(filename).exists() {
        create_db(filename)
    }
    SqliteConnectionManager::file(filename)
}

fn create_db(filename: &str) {
    let db = Connection::open(filename)
        .unwrap_or_else(|_| panic!(format!("Unable to open database file {}", filename)));

    db.execute_batch(include_str!("document_schema.sql"))
        .expect("Failed to execute document schema.");
    db.execute_batch(include_str!("user_schema.sql"))
        .expect("Failed to execute user schema.");
    db.execute_batch(include_str!("setting_schema.sql"))
        .expect("Failed to execute setting schema.");

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
