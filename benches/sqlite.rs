/**
 * What did I learn from this benchmark?
 *
 * 1. Default sqlite is twice as fast when using write-ahead logging, also for reading.
 * 2. Lookup time is about the same for repeating and random primary keys - there is no cache.
 * 3. Filling a db is best done when using a transaction.
 * 4. Unprepared simple get-statements take ~50% (8 us) extra time vs prepared statement.
 * 5. Preparing same statement many times takes only ~2.5 us, prepared statements are cached?
 *    Why do unprepared query take 8 us more time?
 *    Should not unprepared queries get the same benefit?
 */
use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use rusqlite::{Connection, Statement, NO_PARAMS};

criterion_group!(benches, sqlite_benchmark);
criterion_main!(benches);

const NUMBER_OF_ROWS: u32 = 10_000;
const DB_FILENAME: &str = "storage/bench.sqlite";

fn sqlite_benchmark(c: &mut Criterion) {
    c.bench_function("get random id", |b| {
        // make sure time(generating random id) <<< time(get random row)
        let mut rng = create_rng();
        b.iter(|| get_random_id(&mut rng))
    });

    c.bench_function("prepare statement", |b| {
        let db = create_filled_db(NUMBER_OF_ROWS);
        b.iter(|| db.prepare("SELECT value FROM bench WHERE id = ?1"))
    });

    c.bench_function("DEFAULT - get with unprepared statement", |b| {
        let db = create_filled_db(NUMBER_OF_ROWS);

        b.iter(|| {
            let _value: u32 = db
                .query_row("SELECT value FROM bench WHERE id = ?1", &[123], |r| {
                    r.get(0)
                })
                .unwrap();
        })
    });

    c.bench_function("DEFAULT - get same row", |b| {
        let db = create_filled_db(NUMBER_OF_ROWS);
        let mut get_value = get_value_statement(&db);
        let mut rng = create_rng();

        b.iter(|| {
            run_query(&mut get_value, &mut rng, |x| {
                get_random_id(x);
                5431
            })
        })
    });

    c.bench_function("DEFAULT - get random row", |b| {
        let db = create_filled_db(NUMBER_OF_ROWS);
        let mut get_value = get_value_statement(&db);
        let mut rng = create_rng();

        b.iter(|| run_query(&mut get_value, &mut rng, get_random_id))
    });

    c.bench_function("WAL - get same row", |b| {
        let db = create_filled_db(NUMBER_OF_ROWS);
        enable_write_ahead_logging(&db);
        let mut get_value = get_value_statement(&db);
        let mut rng = create_rng();

        b.iter(|| {
            run_query(&mut get_value, &mut rng, |x| {
                get_random_id(x);
                5431
            })
        })
    });

    c.bench_function("WAL - get random row", |b| {
        let db = create_filled_db(NUMBER_OF_ROWS);
        enable_write_ahead_logging(&db);
        let mut get_value = get_value_statement(&db);
        let mut rng = create_rng();

        b.iter(|| run_query(&mut get_value, &mut rng, get_random_id))
    });
}

fn create_filled_db(number_of_rows: u32) -> Connection {
    std::fs::remove_file(DB_FILENAME).unwrap_or(());

    let mut db = Connection::open(DB_FILENAME).unwrap();
    create_bench_table(&db);
    fill_db(&mut db, number_of_rows);

    db
}

fn create_bench_table(db: &Connection) {
    db.execute(
        "
      CREATE TABLE bench (
        id INTEGER PRIMARY KEY,
        value INTEGER
      )
  ",
        NO_PARAMS,
    )
    .unwrap();
}

fn fill_db(db: &mut Connection, number_of_rows: u32) {
    // transaction speeds up insertion
    let transaction = db.transaction().unwrap();
    {
        // drop `insert` before commiting
        let mut insert = transaction
            .prepare("INSERT INTO bench (id, value) values (?1, ?2)")
            .unwrap();
        for i in 0..number_of_rows {
            insert.execute(&[i, 42]).unwrap();
        }
    }
    transaction.commit().unwrap();
}

fn get_value_statement(db: &Connection) -> Statement {
    db.prepare("SELECT value FROM bench WHERE id = ?1").unwrap()
}

fn enable_write_ahead_logging(db: &Connection) {
    // PRAGMA journal_mode=wal;
    let result: String = db
        .pragma_update_and_check(None, "journal_mode", &"wal", |row| row.get(0))
        .unwrap();
    assert!("wal" == &result);
}

fn run_query<F>(statement: &mut Statement, rng: &mut SmallRng, get_id: F) -> u32
where
    F: Fn(&mut SmallRng) -> u32,
{
    let id = get_id(rng);
    statement.query_row(&[id], |r| r.get(0)).unwrap()
}

fn create_rng() -> SmallRng {
    SmallRng::from_seed([1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 2, 3, 4, 5, 6, 7])
}

fn get_random_id(rng: &mut SmallRng) -> u32 {
    rng.gen_range(0, NUMBER_OF_ROWS)
}
