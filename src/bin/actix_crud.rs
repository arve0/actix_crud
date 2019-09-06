use actix_crud::{db, document, user};
use actix_files::{Files, NamedFile};
use actix_session::CookieSession;
use actix_web::{middleware, web, App, HttpServer};

fn main() -> Result<(), failure::Error> {
    // enable logging with RUST_LOG=info
    env_logger::init();
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let server = HttpServer::new(move || {
        App::new()
            .data(db::get_pool())
            .wrap(middleware::Logger::default())
            .wrap(
                CookieSession::signed(&[0; 32]) // TODO: signing key
                    .secure(false),
            )
            .route("/", web::get().to(index))
            .service(Files::new("/static", "client/public/static"))
            .configure(user::config)
            .configure(document::config)
    })
    .bind(["0.0.0.0:", &port].concat())?;

    println!("Listening on port {}", &port);
    server.run().map_err(failure::Error::from)
}

fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("client/public/index.html")
}
