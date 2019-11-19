use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{named_params, Error as SqliteError, Row};
use serde_derive::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::sync::Mutex;
use uuid::Uuid;

use crate::db::{Pool, PooledConnection};
use crate::updates::ClientUpdates;
use crate::user::AuthorizedUser;
use crate::Error;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/document")
            .service(
                web::resource("")
                    .route(web::post().to(insert))
                    .route(web::get().to(get_all)),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get))
                    .route(web::delete().to(delete))
                    .route(web::put().to(update)),
            ),
    );
}

fn insert(
    document: web::Json<JSON>,
    login: AuthorizedUser,
    pool: web::Data<Pool>,
    updates: web::Data<Mutex<ClientUpdates>>,
) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let entry = DBEntry::new(document.into_inner(), &login);
    entry.insert(db)?;

    let document = Document {
        id: entry.id,
        data: entry.data,
    };

    updates.lock().unwrap().inserted(&document, &login);

    Ok(HttpResponse::Created().body(document.id))
}

fn get_all(login: AuthorizedUser, pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let entries = DBEntry::get_all(&login.username, db).map_err(|err| match err {
        SqliteError::QueryReturnedNoRows => ErrorNotFound("not found"),
        err => ErrorInternalServerError(err),
    })?;

    Ok(HttpResponse::Ok().json(entries))
}

fn get(
    id: web::Path<String>,
    login: AuthorizedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let entry =
        DBEntry::get_by_id(id.into_inner(), &login.username, db).map_err(|err| match err {
            SqliteError::QueryReturnedNoRows => ErrorNotFound("not found"),
            err => ErrorInternalServerError(err),
        })?;

    Ok(HttpResponse::Ok().json(entry))
}

fn update(
    id: web::Path<String>,
    data: web::Json<JSON>,
    login: AuthorizedUser,
    pool: web::Data<Pool>,
    updates: web::Data<Mutex<ClientUpdates>>,
) -> Result<HttpResponse, Error> {
    let db = pool.get()?;

    let id = id.into_inner();
    let username = login.username.clone();
    let data = data.into_inner();

    if DBEntry::exists(&id, &username, &db)? {
        DBEntry::update(&id, &username, &data, db)?;

        let document = Document { id, data };

        updates.lock().unwrap().updated(&document, &login);

        Ok(HttpResponse::Ok().body("updated"))
    } else {
        Err(ErrorNotFound([&id, " not found"].concat()).into())
    }
}

fn delete(
    id: web::Path<String>,
    login: AuthorizedUser,
    pool: web::Data<Pool>,
    updates: web::Data<Mutex<ClientUpdates>>,
) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let id = id.into_inner();
    let deleted = DBEntry::delete(&id, &login.username, db)?;

    if deleted {
        updates.lock().unwrap().deleted(&id, &login);

        Ok(HttpResponse::Ok().body(r#"deleted"#))
    } else {
        Ok(HttpResponse::NotFound().body(r#"not found"#))
    }
}

type DBResult<T> = Result<T, SqliteError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    id: String,
    data: JSON,
}

impl Document {
    pub fn from_row(row: &Row) -> DBResult<Self> {
        Ok(Self {
            id: row.get(0)?,
            data: row.get(1)?,
        })
    }
}

struct DBEntry {
    id: String,
    created: i64,
    username: String,
    data: JSON,
}

impl DBEntry {
    pub fn new(document: JSON, login: &AuthorizedUser) -> Self {
        Self {
            id: format!("{}", Uuid::new_v4()),
            created: Utc::now().timestamp(),
            username: login.username.clone(),
            data: document,
        }
    }

    pub fn get_all(username: &str, db: PooledConnection) -> DBResult<Vec<Document>> {
        db.prepare_cached("select id, data from document where username = :username order by created desc limit 100")?
            .query_map_named(
                named_params! { ":username": username, },
                Document::from_row
            )?
            .collect()
    }

    pub fn get_by_id(id: String, username: &str, db: PooledConnection) -> DBResult<Document> {
        db.prepare_cached("select id, data from document where id=:id and username=:username")?
            .query_row_named(
                named_params! {
                    ":id": &id,
                    ":username": &username,
                },
                Document::from_row,
            )
    }

    pub fn insert(&self, db: PooledConnection) -> DBResult<()> {
        let number_of_rows = db
            .prepare_cached("insert into document (id, created, updated, username, data) values (:id, :created, :updated, :username, :data)")?
            .execute_named(named_params! {
                ":id": self.id,
                ":created": self.created,
                ":updated": self.created,
                ":username": self.username,
                ":data": self.data,
            })?;

        assert!(number_of_rows == 1);
        Ok(())
    }

    pub fn update(id: &str, username: &str, data: &JSON, db: PooledConnection) -> DBResult<()> {
        let number_of_rows = db
            .prepare_cached("update document set data=:data, updated=:updated where id=:id and username=:username")?
            .execute_named(named_params! {
                ":id": &id,
                ":updated": Utc::now().timestamp(),
                ":username": &username,
                ":data": data,
            })?;
        assert!(number_of_rows == 1);
        Ok(())
    }

    fn exists(id: &str, username: &str, db: &PooledConnection) -> DBResult<bool> {
        db.prepare_cached("select count(*) from document where id=:id and username=:username")?
            .query_row_named(
                named_params! {
                    ":id": id,
                    ":username": username,
                },
                |row| row.get(0),
            )
            .map(|count: i64| count != 0)
    }

    pub fn delete(id: &str, username: &str, db: PooledConnection) -> DBResult<bool> {
        db.prepare_cached("delete from document where id=:id and username=:username")?
            .execute_named(named_params! {
                ":id": id,
                ":username": username,
            })
            .map(|count| count != 0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
