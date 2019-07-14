use r2d2_sqlite;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, NO_PARAMS};
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

    db.execute(include_str!("db/schema.sql"), NO_PARAMS)
        .expect("Unable to create table in database.");

    enable_write_ahead_logging(&db);
}

fn enable_write_ahead_logging(db: &Connection) {
    // PRAGMA journal_mode=wal;
    let result: String = db
        .pragma_update_and_check(None, "journal_mode", &"wal", |row| row.get(0))
        .unwrap();
    assert!("wal" == &result);
}
