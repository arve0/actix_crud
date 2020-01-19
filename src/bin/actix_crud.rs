use actix_crud::{db, document, setting, updates, user};
use actix_files::{Files, NamedFile};
use actix_session::CookieSession;
use actix_web::cookie::SameSite;
use actix_web::{http::header, middleware, web, App, HttpRequest, HttpServer, Responder};

fn main() -> Result<(), failure::Error> {
    // enable logging with RUST_LOG=info
    env_logger::init();
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let db_pool = db::get_pool();
    let cookie_session_key = setting::get_or_create_cookie_session_key(&db_pool.get()?);

    let client_updates = updates::ClientUpdates::create();

    let server = HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .register_data(client_updates.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                CookieSession::signed(&cookie_session_key)
                    .secure(false) // allow http (vs https)
                    .http_only(true) // disallow access from JS
                    .same_site(SameSite::Lax), // disallow CSRF
            )
            .route("/", web::get().to(index))
            .service(Files::new("/static", "client/public/static"))
            .configure(user::config)
            .configure(document::config)
            .configure(updates::config)
    })
    .bind(["0.0.0.0:", &port].concat())?;

    println!("Listening on port {}", &port);
    server.run().map_err(failure::Error::from)
}

fn index(req: HttpRequest) -> impl Responder {
    NamedFile::open("client/public/index.html")
        .unwrap()
        .with_header(
            header::SET_COOKIE,
            user::AuthorizedUser::logged_in_cookie(&req).to_string(),
        )
}
