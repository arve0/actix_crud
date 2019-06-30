use actix_web::{middleware, web, App, HttpResponse, HttpServer, ResponseError};
use r2d2_sqlite;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, Error as SqliteError, Row, NO_PARAMS};
use serde_derive::{Deserialize, Serialize};
use std::path::Path;

type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

fn main() -> Result<(), failure::Error> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let server = HttpServer::new(move || {
        App::new()
            .data(get_db_pool())
            .wrap(middleware::Logger::default())
            .service(web::resource("/{id}").route(web::get().to(get)))
        // .service(post)
        // .service(put)
        // .service(delete)
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
    Connection::open(DB_FILENAME)
        .expect("Unable to open database.")
        .execute(include_str!("db/schema.sql"), NO_PARAMS)
        .expect("Unable to create table in database.");
}

fn get(id: web::Path<String>, pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let mut get_by_id = db
        .prepare_cached(include_str!("db/get_entry_by_id.sql"))
        .expect("Unable to parse db/get_entry_by_id.sql");

    let result = get_by_id.query_row(&[id.into_inner()], DBEntry::from_row)?;

    Ok(HttpResponse::Ok().json(result))
}

#[derive(Debug, Serialize, Deserialize)]
struct DBEntry {
    id: String,
    revision: i64,
    hash: Vec<u8>,
    prev_hash: Option<Vec<u8>>,
    data: Vec<u8>,
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
