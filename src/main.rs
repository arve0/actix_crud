/* Actix-Web Asynchronous Database Example

This project illustrates two examples:

    1. An asynchronous handler that executes 4 queries in *sequential order*,
       collecting the results and returning them as a single serialized json object

    2. An asynchronous handler that executes 4 queries in *parallel*,
       collecting the results and returning them as a single serialized json object
 */
use actix_web::{middleware, web, App, Error as AWError, HttpResponse, HttpServer};
use futures::future::{join_all, Future};
use r2d2_sqlite;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{NO_PARAMS};
use failure::{err_msg, Error};
// use serde_derive::{Deserialize, Serialize};

mod db;
use db::{Pool, Connection, Queries};

fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // Start N db executor actors (N = number of cores avail)
    let manager = SqliteConnectionManager::file("sakkosekk.db");
    let pool = Pool::new(manager).unwrap();

    let connection = pool.get().map_err(Error::from)?;
    setup_db(connection)?;

    // Start http server
    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            // .service(web::resource("/get").route(web::get().to_async(get_entries)))
            .service(
                web::resource("/parallel_weather").route(web::get().to_async(parallel_weather)),
            )
    })
    .bind("127.0.0.1:8080")?;

    println!("Started http server: 127.0.0.1:8080");
    server.run().map_err(Error::from)
}

fn setup_db(connection: Connection) -> Result<(), Error> {
    let schema = include_str!("schema.sql");
    let db_schema: Result<String, rusqlite::Error> = connection
        .prepare(include_str!("get_schema.sql"))?
        .query_row(NO_PARAMS, |row| row.get(0));

    match db_schema {
        Ok(ref current_schema) if current_schema == schema => Ok(()),
        Ok(_) => Err(err_msg("Schema mismatch. Recreate or migrate db.")),
        _ => {
            println!("Creating table sakkosekk from schema.sql");
            connection
                .prepare(schema)?
                .execute(NO_PARAMS)
                .map_err(Error::from)
                .map(|_| ())
        }
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// struct DBEntry {
//     id: u64,
//     revision: u64,
//     hash: Vec<u8>,
//     prev_hash: Vec<u8>,
//     json: String,
// }

// fn get_entries(db: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = AWError> {
//     execute(&db)
//         .map_err(AWError::from)
//         .and_then(|result| HttpResponse::Ok().json(result))
// }

// /// Version 1: Calls 4 queries in sequential order, as an asynchronous handler
// fn execute(pool: &Pool) -> impl Future<Item = Vec<DBEntry>, Error = AWError> {
//     // TODO: use sqlite WAL and only block on writes
//     let pool = pool.clone();
//     web::block(move || {
//         fut_ok(
//             really_get(
//                 pool.get()?
//             )?
//         )
//     }).from_err()
// }

// fn really_get(connection: Connection) -> Result<Vec<DBEntry>, AWError> {
//     connection.prepare(include_str!("get_db_entries.sql"))?
//         .query_map(NO_PARAMS, into_db_entry)
//         .map_err(Error::from)
//         .and_then(Iterator::collect)
// }

// fn into_db_entry(row: &Row) -> DBEntry {
//     DBEntry {
//         id: row.get(0),
//         revision: 0,
//         hash: vec![],
//         prev_hash: vec![],
//         json: String::new(),
//     }
// }

/// Version 2: Calls 4 queries in parallel, as an asynchronous handler
/// Returning Error types turn into None values in the response
fn parallel_weather(db: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = AWError> {
    let fut_result = vec![
        Box::new(db::execute(&db, Queries::GetTopTenHottestYears)),
        Box::new(db::execute(&db, Queries::GetTopTenColdestYears)),
        Box::new(db::execute(&db, Queries::GetTopTenHottestMonths)),
        Box::new(db::execute(&db, Queries::GetTopTenColdestMonths)),
    ];

    join_all(fut_result)
        .map_err(AWError::from)
        .map(|result| HttpResponse::Ok().json(result))
}
