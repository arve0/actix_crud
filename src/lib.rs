use actix_web::error::{Error as WebError, ErrorConflict, ErrorInternalServerError};
use actix_web::{HttpResponse, ResponseError};
use bcrypt::BcryptError;
use db::is_primary_key_constraint;
use rusqlite::Error as SqliteError;

pub mod db;
pub mod document;
pub mod setting;
pub mod updates;
pub mod user;

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

#[test]
fn test_init_service() {
    use actix_web::dev::Service;
    use actix_web::{http::StatusCode, test, web, App, HttpResponse};

    let mut app =
        test::init_service(App::new().service(web::resource("/test").to(|| HttpResponse::Ok())));

    // Create request object
    let req = test::TestRequest::with_uri("/test").to_request();

    // Execute application
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
