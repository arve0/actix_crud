use rusqlite::{named_params, Error as SqliteError, Row};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde_derive::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::db::PooledConnection;

#[derive(Debug, Serialize, Deserialize)]
pub struct DBEntry {
    id: String,
    data: JSON,
}

impl DBEntry {
    pub fn from_row(row: &Row) -> Result<Self, SqliteError> {
        Ok(Self {
            id: row.get(0)?,
            data: row.get(1)?,
        })
    }

    pub fn get_by_id(db: PooledConnection, id: String) -> Result<Self, SqliteError> {
        db.prepare_cached(include_str!("get_by_id.sql"))?
            .query_row(&[id], Self::from_row)
    }

    pub fn insert(&self, db: PooledConnection) -> Result<(), SqliteError> {
        let number_of_rows = db
            .prepare_cached(include_str!("insert.sql"))?
            .execute_named(named_params! {
                ":id": self.id,
                ":data": self.data,
            })?;

        assert!(number_of_rows == 1);
        Ok(())
    }

    pub fn update(&self, db: PooledConnection) -> Result<(), SqliteError> {
        if self.exists(&db)? {
            let number_of_rows = db
                .prepare_cached(include_str!("update.sql"))?
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
        db.prepare_cached(include_str!("count_by_id.sql"))?
            .query_row(&[&self.id], |row| row.get(0))
            .map(|count: i64| count != 0)
    }

    pub fn delete(db: PooledConnection, id: String) -> Result<bool, SqliteError> {
        db.prepare_cached(include_str!("delete_by_id.sql"))?
            .execute(&[id])
            .map(|count| count != 0)
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
