use actix_crud;
use actix_web::{middleware, App, HttpServer};

fn main() -> Result<(), failure::Error> {
    // enable logging with RUST_LOG=info
    env_logger::init();

    let crud = actix_crud::App::create();
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(|cfg| crud.clone().config(cfg))
    })
    .bind(["0.0.0.0:", &port].concat())?;

    println!("Listening on port {}", &port);
    server.run().map_err(failure::Error::from)
}
