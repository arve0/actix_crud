use actix_session::CookieSession;
use actix_web::error::{Error as WebError, ErrorConflict, ErrorInternalServerError};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, ResponseError};
use bcrypt::BcryptError;
use rusqlite::Error as SqliteError;

mod db;
mod document;
mod user;

use db::is_primary_key_constraint;

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

/**
 * Wrap actix_web::Error inside our own error type, such
 * that we can implement the From trait and avoid
 * Result::map_err all over the place.
 */
#[derive(Debug)]
pub struct Error(WebError);

impl From<r2d2::Error> for Error {
    fn from(error: r2d2::Error) -> Self {
        Error(ErrorInternalServerError(error))
    }
}

impl From<SqliteError> for Error {
    fn from(error: SqliteError) -> Self {
        if is_primary_key_constraint(&error) {
            Self(ErrorConflict("conflict"))
        } else {
            Self(ErrorInternalServerError(error))
        }
    }
}

impl From<WebError> for Error {
    fn from(error: WebError) -> Self {
        Self(error)
    }
}

impl From<BcryptError> for Error {
    fn from(error: BcryptError) -> Self {
        Self(ErrorInternalServerError(error))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        self.0.as_response_error().error_response()
    }
}
