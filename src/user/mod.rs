use crate::db::{Pool, PooledConnection};
use crate::Error;
use actix_session::Session;
use actix_web::dev::Payload;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{web, FromRequest, HttpRequest};
use bcrypt::{hash, verify};
use rusqlite::Error as SqliteError;
use rusqlite::Error::QueryReturnedNoRows;
use rusqlite::{named_params, Row};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

const PASSWORD_HASH_COST: u32 = 4;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .route("/register", web::to(register))
            .route("/login", web::to(login))
            .route("/logout", web::to(logout)),
    );
}

fn register(
    user: web::Form<User>,
    login: UnauthorizedUser,
    pool: web::Data<Pool>,
) -> Result<&'static str, Error> {
    let db = pool.get()?;
    user.create(&db)?;
    login.create_session(&user.username, &db)?;
    Ok("user created")
}

fn login(
    user: web::Form<User>,
    login: UnauthorizedUser,
    pool: web::Data<Pool>,
) -> Result<&'static str, Error> {
    let db = pool.get()?;
    if user.verify_password(&db)? {
        login.create_session(&user.username, &db)?;
        Ok("logged in")
    } else {
        Err(Error(ErrorUnauthorized("wrong username or password")))
    }
}

fn logout(login: AuthorizedUser, pool: web::Data<Pool>) -> Result<&'static str, Error> {
    let db = pool.get()?;
    login.delete_session(&db)?;
    Ok("logged out")
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

    fn create(&self, db: &PooledConnection) -> Result<(), Error> {
        let number_of_rows = db
            .prepare_cached(include_str!("insert.sql"))?
            .execute_named(named_params! {
                ":username": self.username,
                ":password": hash(&self.password, PASSWORD_HASH_COST)?,
            })?;

        assert!(number_of_rows == 1);
        Ok(())
    }

    fn verify_password(&self, db: &PooledConnection) -> Result<bool, Error> {
        db.prepare_cached(include_str!("get_by_username.sql"))?
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
            .prepare_cached(include_str!("insert_session.sql"))?
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
    uuid: Uuid,
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
                    .prepare_cached(include_str!("get_session_username.sql"))?
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
        db.prepare_cached(include_str!("delete_session.sql"))?
            .execute_named(named_params! {
                ":key": self.uuid
            })?;
        self.session.remove("session");
        Ok(())
    }
}
