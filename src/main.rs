use actix_web::error::{Error as WebError, ErrorConflict, ErrorInternalServerError};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, ResponseError};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{named_params, Error as SqliteError, Row};
use serde_derive::{Deserialize, Serialize};
use serde_json::value::RawValue;

mod db;
use db::{Pool, PooledConnection};

fn main() -> Result<(), failure::Error> {
    // enable logging with RUST_LOG=info
    env_logger::init();

    let server = HttpServer::new(move || {
        App::new()
            .data(db::get_pool("entries.sqlite"))
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get))
                    .route(web::delete().to(delete)),
            )
            .service(
                web::resource("/")
                    .route(web::post().to(insert))
                    .route(web::put().to(update)),
            )
    })
    .bind("127.0.0.1:8080")?;

    println!("Started http server: 127.0.0.1:8080");
    server.run().map_err(failure::Error::from)
}

fn get(id: web::Path<String>, pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let entry = DBEntry::get_by_id(db, id.into_inner())?;

    Ok(HttpResponse::Ok().json(entry))
}

fn insert(entry: web::Json<DBEntry>, pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    entry.insert(db)?;

    Ok(HttpResponse::Created().body(r#"{"ok":true}"#))
}

fn update(entry: web::Json<DBEntry>, pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    entry.update(db)?;

    Ok(HttpResponse::Created().body(r#"{"ok":true}"#))
}

fn delete(id: web::Path<String>, pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let deleted = DBEntry::delete(db, id.into_inner())?;

    if deleted {
        Ok(HttpResponse::Ok().body(r#"{"ok":true}"#))
    } else {
        Ok(HttpResponse::NotFound().body(r#"{"ok":false,"error":"Document not found"}"#))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DBEntry {
    id: String,
    data: JSON,
}

impl DBEntry {
    fn from_row(row: &Row) -> Result<Self, SqliteError> {
        Ok(Self {
            id: row.get(0)?,
            data: row.get(1)?,
        })
    }

    fn get_by_id(db: PooledConnection, id: String) -> Result<Self, SqliteError> {
        db.prepare_cached(include_str!("db/get_entry_by_id.sql"))?
            .query_row(&[id], Self::from_row)
    }

    fn insert(&self, db: PooledConnection) -> Result<(), SqliteError> {
        let number_of_rows = db
            .prepare_cached(include_str!("db/insert.sql"))?
            .execute_named(named_params! {
                ":id": self.id,
                ":data": self.data,
            })?;

        assert!(number_of_rows == 1);
        Ok(())
    }

    fn update(&self, db: PooledConnection) -> Result<(), SqliteError> {
        if self.exists(&db)? {
            let number_of_rows = db
                .prepare_cached(include_str!("db/update.sql"))?
                .execute_named(named_params! {
                    ":id": self.id,
                    ":data": self.data,
                })?;
            assert!(number_of_rows == 1);
            Ok(())
        } else {
            self.insert(db)
        }
    }

    fn exists(&self, db: &PooledConnection) -> Result<bool, SqliteError> {
        db.prepare_cached(include_str!("db/count_by_id.sql"))?
            .query_row(&[&self.id], |row| row.get(0))
            .map(|count: i64| count != 0)
    }

    fn delete(db: PooledConnection, id: String) -> Result<bool, SqliteError> {
        db.prepare_cached(include_str!("db/delete_by_id.sql"))?
            .execute(&[id])
            .map(|count| count != 0)
    }
}

/**
 * Wrap actix_web::Error inside our own error type, such
 * that we can implement the From trait and avoid
 * Result::map_err all over the place.
 */
#[derive(Debug)]
struct Error(WebError);

impl From<r2d2::Error> for Error {
    fn from(error: r2d2::Error) -> Self {
        Error(ErrorInternalServerError(error))
    }
}

impl From<SqliteError> for Error {
    fn from(error: SqliteError) -> Self {
        use libsqlite3_sys::Error as LibSqliteError;
        use libsqlite3_sys::ErrorCode::ConstraintViolation;
        use rusqlite::Error::SqliteFailure;

        match error {
            // when inserting and document exists
            SqliteFailure(
                LibSqliteError {
                    code: ConstraintViolation,
                    extended_code: 1555,
                },
                Some(_),
            ) => Self(ErrorConflict("document already exists")),
            error => Self(ErrorInternalServerError(error)),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        self.0.as_response_error().error_response()
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
