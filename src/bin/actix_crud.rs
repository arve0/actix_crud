use actix_crud::{db, document, user};
use actix_session::CookieSession;
use actix_web::{middleware, web, App, HttpServer};

fn main() -> Result<(), failure::Error> {
    // enable logging with RUST_LOG=info
    env_logger::init();

    let server = HttpServer::new(move || {
        App::new()
            .data(db::get_pool())
            .wrap(middleware::Logger::default())
            .wrap(
                CookieSession::signed(&[0; 32]) // TODO: signing key
                    .secure(false),
            )
            .route("/", web::get().to(|| ""))
            .configure(user::config)
            .configure(document::config)
    })
    .bind("127.0.0.1:8080")?;

    println!("Started http server: 127.0.0.1:8080");
    server.run().map_err(failure::Error::from)
}
