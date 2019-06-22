#[macro_use]
extern crate criterion;

use criterion::Criterion;
// use criterion::black_box;

use rusqlite::{Connection, Statement, NO_PARAMS, Result};
use std::io::Write;
use rand::prelude::*;

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn criterion_benchmark(c: &mut Criterion) {
  std::fs::remove_file("bench.sqlite").unwrap_or(());
  let mut connection = Connection::open("bench.sqlite").unwrap();
  connection.execute("
      CREATE TABLE bench (
        id INTEGER PRIMARY KEY,
        value INTEGER
      )
  ", NO_PARAMS).unwrap();


  let transaction = connection.transaction().unwrap();
  let mut rng = rand::thread_rng();
  fill(&transaction, &mut rng).unwrap();
  transaction.commit().unwrap();
  println!("");


  // c.sample_size(10);
  c.bench_function("regular read", |b| {
    let get_all_values = "SELECT value FROM bench WHERE id >= ?1 and id < ?2";
    let connection = Connection::open("bench.sqlite").unwrap();
    let mut rng = rand::thread_rng();
    let mut statement = connection.prepare(get_all_values).unwrap();
    b.iter(|| run(&mut statement, &mut rng))
  });
}

fn run(statement: &mut Statement, rng: &mut ThreadRng) -> usize {
  let n: u32 = rng.gen_range(0, 1000_000);
  let rows: Result<Vec<u32>> = statement.query_map(&[n, n + 100], |r| r.get(0)).unwrap().collect();
  rows.unwrap().len()
}

fn fill(connection: &Connection, rng: &mut ThreadRng) -> Result<()> {
  let mut insert = connection.prepare("INSERT INTO bench (id, value) values (?1, ?2)")?;
  print!("filling database");
  for i in 0..1000_000 {
    if (i + 1) % 10_000 == 0 {
      print!(".");
      std::io::stdout().flush().unwrap_or(());
    }
    insert.execute(&[i, rng.gen::<u32>()])?;
  }
  Ok(())
}
