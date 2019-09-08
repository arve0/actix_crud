use actix_crud::{db, document, user, setting};
use actix_files::{Files, NamedFile};
use actix_session::CookieSession;
use actix_web::{middleware, web, App, HttpServer};
use actix_web::cookie::{SameSite};

fn main() -> Result<(), failure::Error> {
    // enable logging with RUST_LOG=info
    env_logger::init();
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let db_pool = db::get_pool();
    let cookie_session_key = setting::get_or_create_cookie_session_key(&db_pool.get()?);

    let server = HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                CookieSession::signed(&cookie_session_key)
                    .secure(false) // allow http (vs https)
                    .http_only(true) // disallow access fra JS
                    .same_site(SameSite::Lax), // disallow CSRF
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
