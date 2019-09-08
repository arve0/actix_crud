use crate::db::PooledConnection;
use rusqlite::NO_PARAMS;
use rand::{thread_rng, Rng};

pub fn get_or_create_cookie_session_key(db: &PooledConnection) -> Vec<u8> {
    db.prepare("select value from setting where name = 'cookie_session_key'")
        .unwrap()
        .query_row(NO_PARAMS, |row| row.get(0))
        .unwrap_or_else(|_| create_cookie_session_key(&db))
}

fn create_cookie_session_key(db: &PooledConnection) -> Vec<u8> {
    let mut key = vec![0u8; 32];
    thread_rng().fill(&mut key[..]);

    let rows = db.prepare("insert into setting values ( 'cookie_session_key', ?)")
        .unwrap()
        .execute(&[&key])
        .expect("Unable to insert cookie session key.");

    assert_eq!(rows, 1, "Cookie session key not stored in database.");

    key
}
