use actix_web::error::{ErrorConflict, ErrorInternalServerError, ErrorNotFound};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{named_params, params, Error as SqliteError, Row};
use serde_derive::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::collections::HashMap;
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
                    .route(web::put().to(update))
                    .route(web::post().to(insert_idempotent)),
            ),
    );
}

fn insert(
    data: web::Json<JSON>,
    login: AuthorizedUser,
    pool: web::Data<Pool>,
    updates: web::Data<Mutex<ClientUpdates>>,
) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let document = Document::insert(data.into_inner(), &login, db)?;

    updates.lock().unwrap().inserted(&document, &login);

    Ok(HttpResponse::Created().json(document))
}

fn get_all(
    parameters: web::Query<HashMap<String, i64>>,
    login: AuthorizedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let below_pk = parameters.get("below_pk").unwrap_or(&i64::max_value());
    let db = pool.get()?;
    let entries = Document::get_all_below_pk(*below_pk, &login, &db).map_err(|err| match err {
        SqliteError::QueryReturnedNoRows => ErrorNotFound("not found"),
        err => ErrorInternalServerError(err),
    })?;

    let mut response = HttpResponse::Ok();

    if let Some(last) = entries.last() {
        if Document::count_below(last.pk, &login, &db) > 0 {
            response.set_header("Link-Next", format!("/document?below_pk={}", last.pk));
        }
    }

    if let Some(first) = entries.first() {
        if Document::count_above(first.pk, &login, &db) > 0 {
            response.set_header("Link-Prev", format!("/document?above_pk={}", first.pk));
        }
    }

    Ok(response.json(entries))
}

fn get(
    id: web::Path<String>,
    login: AuthorizedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    let entry = Document::get_by_id(&id.into_inner(), &login, &db).map_err(|err| match err {
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
    let data = data.into_inner();

    if let Ok(metadata) = Document::get_by_id_without_data(&id, &login, &db) {
        let document = Document::update(metadata, data, db)?;

        updates.lock().unwrap().updated(&document, &login);

        Ok(HttpResponse::Ok().body("updated"))
    } else {
        Err(ErrorNotFound([&id, " not found"].concat()).into())
    }
}

fn insert_idempotent(
    id: web::Path<String>,
    data: web::Json<JSON>,
    login: AuthorizedUser,
    pool: web::Data<Pool>,
    updates: web::Data<Mutex<ClientUpdates>>,
) -> Result<HttpResponse, Error> {
    let db = pool.get()?;

    let id = id.into_inner();
    let data = data.into_inner();

    if let Ok(existing) = Document::get_by_id(&id, &login, &db) {
        let existing_data = existing.data.0.get();
        let new_data = data.0.get();
        if existing_data == new_data {
            Ok(HttpResponse::Created().json(existing))
        } else {
            Err(ErrorConflict([&id, " already exists and data is not the same"].concat()).into())
        }
    } else {
        let document = Document::insert_with_id(id, data, &login, db)?;
        updates.lock().unwrap().inserted(&document, &login);

        Ok(HttpResponse::Created().json(document))
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
    let deleted = Document::delete(&id, &login, db)?;

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
    pk: i64,
    id: String,
    created: i64,
    data: JSON,
}

impl Document {
    pub fn insert(data: JSON, login: &AuthorizedUser, db: PooledConnection) -> DBResult<Document> {
        let id: String = Uuid::new_v4().to_string();
        Self::insert_with_id(id, data, login, db)
    }

    pub fn insert_with_id(
        id: String,
        data: JSON,
        login: &AuthorizedUser,
        db: PooledConnection,
    ) -> DBResult<Document> {
        let created = Utc::now().timestamp();

        let pk = db
            .prepare_cached(
                "insert into document (id, created, updated, username, data)
                values (?, ?, ?, ?, ?)",
            )?
            .insert(params![&id, &created, &created, login.username, &data])?;

        Ok(Self {
            pk,
            id,
            created,
            data,
        })
    }

    pub fn from_row(row: &Row) -> DBResult<Self> {
        Ok(Self {
            pk: row.get(0)?,
            id: row.get(1)?,
            created: row.get(2)?,
            data: row.get(3)?,
        })
    }

    pub fn from_row_without_data(row: &Row) -> DBResult<Self> {
        Ok(Self {
            pk: row.get(0)?,
            id: row.get(1)?,
            created: row.get(2)?,
            data: Default::default(),
        })
    }

    pub fn get_all_below_pk(
        below_pk: i64,
        login: &AuthorizedUser,
        db: &PooledConnection,
    ) -> DBResult<Vec<Document>> {
        db.prepare_cached(
            "select pk, id, created, data from document
            where pk < :pk and username = :username
            order by pk desc limit 100",
        )?
        .query_map_named(
            named_params! {
                ":pk": below_pk,
                ":username": login.username,
            },
            Document::from_row,
        )?
        .collect()
    }

    pub fn count_below(pk: i64, login: &AuthorizedUser, db: &PooledConnection) -> i64 {
        db.prepare_cached(
            "select count(*) from document
            where pk < :pk and username = :username",
        )
        .expect("Unable to create prepared statement.")
        .query_row_named(
            named_params! {
                ":pk": pk,
                ":username": login.username,
            },
            |r| r.get(0),
        )
        .unwrap()
    }

    pub fn count_above(pk: i64, login: &AuthorizedUser, db: &PooledConnection) -> i64 {
        db.prepare_cached(
            "select count(*) from document
            where pk > :pk and username = :username",
        )
        .expect("Unable to create prepared statement.")
        .query_row_named(
            named_params! {
                ":pk": pk,
                ":username": login.username,
            },
            |r| r.get(0),
        )
        .unwrap()
    }

    pub fn get_by_id(
        id: &str,
        login: &AuthorizedUser,
        db: &PooledConnection,
    ) -> DBResult<Document> {
        db.prepare_cached(
            "select pk, id, created, data from document
            where id=:id and username=:username",
        )?
        .query_row_named(
            named_params! {
                ":id": id,
                ":username": login.username,
            },
            Document::from_row,
        )
    }

    pub fn update(metadata: Document, data: JSON, db: PooledConnection) -> DBResult<Document> {
        let updated = Utc::now().timestamp();
        let number_of_rows = db
            .prepare_cached(
                "update document
                set data=:data, updated=:updated
                where pk=:pk",
            )?
            .execute_named(named_params! {
                ":pk": metadata.pk,
                ":updated": &updated,
                ":data": &data,
            })?;

        assert!(number_of_rows == 1);

        Ok(Self {
            pk: metadata.pk,
            id: metadata.id,
            created: metadata.created,
            data,
        })
    }

    fn get_by_id_without_data(
        id: &str,
        login: &AuthorizedUser,
        db: &PooledConnection,
    ) -> DBResult<Document> {
        db.prepare_cached(
            "select pk, id, created, username from document
            where id=:id and username=:username",
        )?
        .query_row_named(
            named_params! {
                ":id": id,
                ":username": login.username,
            },
            Document::from_row_without_data,
        )
    }

    pub fn delete(id: &str, login: &AuthorizedUser, db: PooledConnection) -> DBResult<bool> {
        db.prepare_cached("delete from document where id=:id and username=:username")?
            .execute_named(named_params! {
                ":id": id,
                ":username": login.username,
            })
            .map(|count| count != 0)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct JSON(Box<RawValue>);

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
