use actix_files::{Files, NamedFile};
use actix_session::CookieSession;
use actix_web::cookie::SameSite;
use actix_web::{http::header, web, web::Data, web::ServiceConfig, HttpRequest, Responder};
use std::sync::Mutex;

use crate::{db, document, setting, updates, user};

#[derive(Clone)]
pub struct App {
    db_pool: db::Pool,
    client_updates: Data<Mutex<updates::ClientUpdates>>,
    cookie_session_key: Vec<u8>,
}

impl App {
    pub fn create() -> Self {
        let db_pool = db::get_pool();
        let client_updates = updates::ClientUpdates::create();
        let cookie_session_key = setting::get_or_create_cookie_session_key(
            &db_pool.get().expect("unable to get db connection"),
        );

        Self {
            db_pool,
            client_updates,
            cookie_session_key,
        }
    }

    pub fn config(self, cfg: &mut ServiceConfig) {
        cfg.service(
            web::scope("") // use scope to access `register_data` API
                .data(self.db_pool.clone())
                .register_data(self.client_updates.clone())
                .wrap(
                    CookieSession::signed(&self.cookie_session_key)
                        .secure(false) // allow http (vs https)
                        .http_only(true) // disallow access from JS
                        .same_site(SameSite::Lax), // disallow CSRF
                )
                .route("/", web::get().to(index))
                .service(Files::new("/static", "client/public/static"))
                .configure(user::config)
                .configure(document::config)
                .configure(updates::config),
        );
    }
}

fn index(req: HttpRequest) -> impl Responder {
    NamedFile::open("client/public/index.html")
        .unwrap()
        .with_header(
            header::SET_COOKIE,
            user::AuthorizedUser::logged_in_cookie(&req).to_string(),
        )
}
