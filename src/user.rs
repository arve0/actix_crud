use actix_session::Session;
use actix_web::dev::Payload;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{web, FromRequest, HttpRequest, HttpResponse};
use bcrypt::{hash, verify};
use rusqlite::Error as SqliteError;
use rusqlite::Error::QueryReturnedNoRows;
use rusqlite::{named_params, Row};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{is_primary_key_constraint, Pool, PooledConnection};
use crate::Error;

const PASSWORD_HASH_COST: u32 = 4;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .route("", web::to(get_username))
            .route("/register", web::to(register))
            .route("/login", web::to(login))
            .route("/logout", web::to(logout)),
    );
}

fn get_username(user: AuthorizedUser) -> String {
    user.username
}

fn register(
    user: web::Form<User>,
    login: UnauthorizedUser,
    pool: web::Data<Pool>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    if user.username.is_empty() {
        return Err(ErrorBadRequest("empty username").into());
    } else if user.password.is_empty() {
        return Err(ErrorBadRequest("empty password").into());
    }

    let db = pool.get()?;
    match user.create(&db) {
        Ok(()) => {
            login.create_session(&user.username, &db)?;

            let referer = get_header(&req, "referer").unwrap_or_else(|| "/");
            Ok(HttpResponse::SeeOther()
                .header("location", referer)
                .body("user registered"))
        }
        Err(error) => {
            if is_primary_key_constraint(&error) {
                Ok(HttpResponse::SeeOther()
                    .header("content-type", "text/html")
                    .body(r#"Unable to register user. <script>setTimeout(function () { history.back() }, 1000)</script>"#))
            } else {
                Err(Error::from(error))
            }
        }
    }
}

fn get_header<'a>(req: &'a HttpRequest, key: &str) -> Option<&'a str> {
    req.headers().get(key)?.to_str().ok()
}

fn login(
    user: web::Form<User>,
    login: UnauthorizedUser,
    pool: web::Data<Pool>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let referer = get_header(&req, "referer").unwrap_or_else(|| "/");

    let db = pool.get()?;
    if user.verify_password(&db)? {
        login.create_session(&user.username, &db)?;
        Ok(HttpResponse::SeeOther()
            .header("location", referer)
            .body("logged in"))
    } else {
        Ok(HttpResponse::Unauthorized()
            .header("content-type", "text/html")
            .body(r#"Wrong username or password. <script>setTimeout(function () { history.back() }, 1000)</script>"#))
    }
}

fn logout(
    login: AuthorizedUser,
    pool: web::Data<Pool>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let db = pool.get()?;
    login.delete_session(&db)?;

    let referer = get_header(&req, "referer").unwrap_or_else(|| "/");
    Ok(HttpResponse::SeeOther()
        .header("location", referer)
        .body("logged out"))
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
}

impl User {
    pub fn from_row(row: &Row) -> Result<Self, SqliteError> {
        Ok(Self {
            username: row.get(0)?,
            password: row.get(1)?,
        })
    }

    fn create(&self, db: &PooledConnection) -> Result<(), SqliteError> {
        let number_of_rows = db
            .prepare_cached("insert into user (username, password) values (:username, :password)")?
            .execute_named(named_params! {
                ":username": self.username,
                ":password": hash(&self.password, PASSWORD_HASH_COST).unwrap(),
            })?;

        assert!(number_of_rows == 1);
        Ok(())
    }

    fn verify_password(&self, db: &PooledConnection) -> Result<bool, Error> {
        db.prepare_cached("select username, password from user where username=?1")?
            .query_row(&[&self.username], Self::from_row)
            .map(|stored| verify(&self.password, &stored.password).unwrap_or_else(|_| false))
            .or_else(|_| Ok(false)) // query_row returned no rows
    }
}

// session cookie is empty -> user not logged in
struct UnauthorizedUser(Session);

impl FromRequest for UnauthorizedUser {
    type Error = Error;
    type Future = Result<Self, Error>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let session = Session::from_request(req, payload)?;
        if let Some(_key) = session.get::<String>("session")? {
            Err(Error(ErrorBadRequest("already logged in")))
        } else {
            Ok(Self(session))
        }
    }
}

impl UnauthorizedUser {
    fn create_session(&self, username: &str, db: &PooledConnection) -> Result<(), Error> {
        let key = Uuid::new_v4();
        let number_of_rows = db
            .prepare_cached("insert into user_session (key, username) values (:key, :username)")?
            .execute_named(named_params! {
                ":key": key,
                ":username": username,
            })?;

        assert!(number_of_rows == 1);
        self.0.set("session", format!("{}", key))?;
        Ok(())
    }
}

// verified from session cookie and persistent storage
pub struct AuthorizedUser {
    pub username: String,
    session: Session,
    pub uuid: Uuid,
}

impl FromRequest for AuthorizedUser {
    type Error = Error;
    type Future = Result<Self, Error>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let session = Session::from_request(req, payload)?;
        if let Some(key) = session.get::<String>("session")? {
            let uuid = Uuid::parse_str(&key).map_err(|_| {
                session.clear();
                ErrorInternalServerError("invalid session")
            })?;
            if let Some(pool) = req.app_data::<Pool>() {
                let username = pool
                    .get()?
                    .prepare_cached("select username from user_session where key=?")?
                    .query_row(&[&uuid], |row| row.get(0))
                    .map_err(|error| match error {
                        QueryReturnedNoRows => {
                            session.clear();
                            Error(ErrorUnauthorized("invalid session"))
                        }
                        error => Error(ErrorInternalServerError(error)),
                    })?;

                Ok(Self {
                    username,
                    session,
                    uuid,
                })
            } else {
                Err(Error(ErrorInternalServerError("unable to connect to db")))
            }
        } else {
            Err(Error(ErrorUnauthorized("unauthorized")))
        }
    }
}

impl AuthorizedUser {
    fn delete_session(&self, db: &PooledConnection) -> Result<(), Error> {
        db.prepare_cached("delete from user_session where key=:key")?
            .execute_named(named_params! {
                ":key": self.uuid
            })?;
        self.session.remove("session");
        Ok(())
    }
}
