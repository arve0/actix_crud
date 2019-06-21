use actix_web::{web, Error as AWError};
use failure::Error;
use futures::Future;
use r2d2;
use r2d2_sqlite;
use rusqlite::{Row, NO_PARAMS};
use serde_derive::{Deserialize, Serialize};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

#[derive(Debug, Serialize, Deserialize)]
pub enum WeatherAgg {
    AnnualAgg { year: i32, total: f64 },
    MonthAgg { year: i32, month: i32, total: f64 },
}

pub enum Queries {
    GetTopTenHottestYears,
    GetTopTenColdestYears,
    GetTopTenHottestMonths,
    GetTopTenColdestMonths,
}

pub fn execute(
    pool: &Pool,
    query: Queries,
) -> impl Future<Item = Vec<WeatherAgg>, Error = AWError> {
    let pool = pool.clone();
    // TODO: use sqlite WAL and only block on writes
    web::block(move || get(query, pool.get()?)).from_err()
}

fn get(query: Queries, conn: Connection) -> Result<Vec<WeatherAgg>, Error> {
    let stmt = match query {
        Queries::GetTopTenHottestYears => include_str!("ten_hottest_years.sql"),
        Queries::GetTopTenColdestYears => include_str!("ten_coldest_years.sql"),
        Queries::GetTopTenHottestMonths => include_str!("ten_hottest_months.sql"),
        Queries::GetTopTenColdestMonths => include_str!("ten_coldest_months.sql"),
    };
    let mut prep_stmt = conn.prepare(stmt)?;
    prep_stmt
        .query_map(NO_PARAMS, into_weather_agg(query))
        .and_then(Iterator::collect)
        .map_err(Error::from)
}

fn into_weather_agg(query: Queries) -> impl FnMut(&Row) -> WeatherAgg {
    use Queries::*;
    move |row| match query {
        GetTopTenHottestYears | GetTopTenColdestYears => WeatherAgg::AnnualAgg {
            year: row.get(0),
            total: row.get(1),
        },
        GetTopTenHottestMonths | GetTopTenColdestMonths => WeatherAgg::MonthAgg {
            year: row.get(0),
            month: row.get(1),
            total: row.get(2),
        },
    }
}
