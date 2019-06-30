use actix_web::{middleware, web, App, HttpResponse, HttpServer, ResponseError};
use r2d2_sqlite;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{named_params, Connection, Error as SqliteError, Row, NO_PARAMS};
use serde_derive::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::path::Path;

type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

fn main() -> Result<(), failure::Error> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let server = HttpServer::new(move || {
        App::new()
            .data(get_db_pool())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get))
                    .route(web::delete().to(delete)),
            )
            .service(web::resource("/").route(web::put().to(put)))
    })
    .bind("127.0.0.1:8080")?;

    println!("Started http server: 127.0.0.1:8080");
    server.run().map_err(failure::Error::from)
}

const DB_FILENAME: &str = "entries.sqlite";

fn get_db_pool() -> Pool {
    Pool::new(get_db_create_if_missing()).expect("Unable to create db pool.")
}

fn get_db_create_if_missing() -> SqliteConnectionManager {
    if !Path::new(DB_FILENAME).exists() {
        create_db()
    }
    SqliteConnectionManager::file(DB_FILENAME)
}

fn create_db() {
    let db = Connection::open(DB_FILENAME).expect("Unable to open database.");

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

fn get(id: web::Path<String>, pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let mut get_by_id = db
        .prepare_cached(include_str!("db/get_entry_by_id.sql"))
        .expect("Unable to parse db/get_entry_by_id.sql");

    let result = get_by_id.query_row(&[id.into_inner()], DBEntry::from_row)?;

    Ok(HttpResponse::Ok().json(result))
}

fn put(entry: web::Json<DBEntry>, pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let mut count_by_id = db
        .prepare_cached(include_str!("db/count_by_id.sql"))
        .expect("Unable to parse db/count_by_id.sql");

    let count: i64 = count_by_id.query_row(&[&entry.id], |row| row.get(0))?;

    if count == 0 {
        let mut insert = db
            .prepare_cached(include_str!("db/insert.sql"))
            .expect("Unable to parse db/insert.sql");
        let number_of_rows = insert.execute_named(named_params! {
            ":id": entry.id,
            ":revision": entry.revision,
            ":hash": entry.hash,
            ":prev_hash": entry.prev_hash,
            ":data": entry.data,
        })?;
        assert!(number_of_rows == 1);
    } else {
        return Ok(HttpResponse::Conflict()
            .body(r#"{"ok":false,"error":"Existing document with same id exists"}"#));
    }

    Ok(HttpResponse::Created().body(r#"{"ok":true}"#))
}

fn delete(id: web::Path<String>, pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let mut delete_by_id = db
        .prepare_cached(include_str!("db/delete_by_id.sql"))
        .expect("Unable to parse db/delete_by_id.sql");

    let affected_rows = delete_by_id.execute(&[id.into_inner()])?;

    if affected_rows == 1 {
        Ok(HttpResponse::Ok().body(r#"{"ok":true}"#))
    } else if affected_rows == 0 {
        return Ok(HttpResponse::NotFound().body(r#"{"ok":false,"error":"Document not found"}"#));
    } else {
        panic!("Removed more then 1 row")
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DBEntry {
    id: String,
    revision: i64,
    hash: Vec<u8>,
    prev_hash: Option<Vec<u8>>,
    data: JSON,
}

impl DBEntry {
    fn from_row(row: &Row) -> Result<Self, SqliteError> {
        Ok(Self {
            id: row.get(0)?,
            revision: row.get(1)?,
            hash: row.get(2)?,
            prev_hash: row.get(3)?,
            data: row.get(4)?,
        })
    }
}

#[derive(Debug)]
enum Error {
    Pool(r2d2::Error),
    Sqlite(SqliteError),
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "asdf"
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<r2d2::Error> for Error {
    fn from(error: r2d2::Error) -> Self {
        Error::Pool(error)
    }
}

impl From<SqliteError> for Error {
    fn from(error: SqliteError) -> Self {
        Error::Sqlite(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Error::*;
        match self {
            Pool(error) => error.fmt(f),
            Sqlite(error) => error.fmt(f),
        }
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        use Error::*;
        match self {
            Sqlite(rusqlite::Error::QueryReturnedNoRows) => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JSON(Box<RawValue>);

impl FromSql for JSON {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(text) => {
                let value = RawValue::from_string(text.to_string())
                    .expect("Got invalid JSON from database");
                Ok(JSON(value))
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for JSON {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        Ok(ToSqlOutput::from(self.0.get()))
    }
}
