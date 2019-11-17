#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

use rocket::outcome::Outcome;
use rocket::http::Status;
use rocket::response::NamedFile;
use rocket::response::status;
use rocket::request::{self, FromRequest, Request};
use rocket::http::{Cookie, Cookies};
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::serve::StaticFiles;

use argonautica::{Hasher, Verifier};
use chrono::{NaiveDateTime, Utc};
use rusqlite::{params, Connection};
use serde_derive::{Deserialize, Serialize};

use std::sync::Mutex;
use std::path::Path;

type DbConn = Mutex<Connection>;

const SECRET_KEY: &str = "TODO: HANDLE SECRET KEY";

/// The error type used throughout the binary
#[derive(Debug, PartialEq)]
enum Error {
    /// An error encountered while working with password hashes
    HashError(argonautica::Error),
    DatabaseErr(rusqlite::Error),
    /// Error returned when an attempt is made to create a new user with a real
    /// name or email address that already exists in the database.
    UserAlreadyExists,
    LoginFailed,
    /// An error occured while trying to launch Rocket
    RocketLaunchErr
}

impl From<argonautica::Error> for Error {
    fn from(err: argonautica::Error) -> Self {
        Error::HashError(err)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::DatabaseErr(err)
    }
}

/// Representation of a user in the database.
// TODO: figure out a strategy for more precisely loading only the user data
// that's really needed
//
// we don't need to be loading the user's password hash on every API request lol
#[derive(Debug, PartialEq)]
struct User {
    /// The user's unique, numeric ID.
    ///
    /// This number starts at 1 (the first user) and is incremented for each
    /// user that signs up.
    user_id: u32,
    /// Argon stores the salt alongside the hash and other info
    hash: String,
    email: String,
    created_at: NaiveDateTime,
    /// The user's display name
    ///
    /// This name can change at any time and is purely cosmetic.
    display_name: String,
    /// The user's real name
    ///
    /// This name can also change (obviously) but should be modified very rarely.
    real_name: String
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, Self::Error> {
        let db = request.guard::<State<DbConn>>().unwrap();
        let conn = db.lock().unwrap();

        let res = request.cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| User::load_id(&conn, id));

        match res {
            Some(Ok(r)) => Outcome::Success(r),
            Some(Err(err)) => Outcome::Failure((Status::InternalServerError, err)),
            None => Outcome::Forward(())
        }
    }
}

impl User {
    /// Creates a table in the given database for storing this struct.
    ///
    /// The table will only be created if it does not already exist.
    fn create_table(conn: &Connection) -> Result<(), Error> {
        Ok(conn.execute(
            "CREATE TABLE if not exists user (
                    user_id                 INTEGER PRIMARY KEY,
                    hash                    TEXT NOT NULL,
                    email                   TEXT NOT NULL,
                    created_at              TEXT NOT NULL,
                    display_name            TEXT NOT NULL,
                    real_name               TEXT NOT NULL
                    )",
            params![],
        ).map(|_| ())?)
    }

    /// Inserts a new user into the database based on the given registration info.
    ///
    /// Errors if the user cannot be created.
    // TODO: error if user already exists with same realname or email
    fn create_new(conn: &Connection, rinfo: &RegisterInfo) -> Result<(), Error> {
        let mut hasher = Hasher::default();
        let hash = hasher
            .with_password(&rinfo.password)
            .with_secret_key(SECRET_KEY)
            .hash()
            .unwrap();
        let created_at = Utc::now().naive_utc();

        Ok(conn.execute(
            "INSERT INTO user (hash, email, created_at, display_name, real_name)
                    VALUES (?1, ?2, ?3, ?4, ?5)",
            params![hash, rinfo.email, created_at, rinfo.display_name, rinfo.real_name],
        ).map(|_| ())?)
    }

    /// Loads the user specified by the given ID from the database.
    fn load_id(conn: &Connection, user_id: u64) -> Result<Self, Error> {
        Ok(conn.query_row(
            &("SELECT user_id, hash, email, created_at, display_name, real_name FROM user WHERE user_id='".to_string() + &user_id.to_string() + "'"),
            params![],
            |row| {
                Ok(User {
                    user_id: row.get(0)?,
                    hash: row.get(1)?,
                    email: row.get(2)?,
                    created_at: row.get(3)?,
                    display_name: row.get(4)?,
                    real_name: row.get(5)?
                })
            }
        )?)
    }

    /// Loads the user specified by the given email from the database
    fn load_email(conn: &Connection, email: &str) -> Result<Self, Error> {
        Ok(conn.query_row(
            &("SELECT user_id, hash, email, created_at, display_name, real_name FROM user WHERE email='".to_string() + email + "'"),
            params![],
            |row| {
                Ok(User {
                    user_id: row.get(0)?,
                    hash: row.get(1)?,
                    email: row.get(2)?,
                    created_at: row.get(3)?,
                    display_name: row.get(4)?,
                    real_name: row.get(5)?
                })
            }
        )?)
    }

    /// Returns true if this user matches the given `LoginInfo`
    fn auth(&self, login_info: &LoginInfo) -> Result<bool, Error> {
        let mut verifier = Verifier::default();
        Ok(
            verifier
                .with_hash(&self.hash)
                .with_password(&login_info.password)
                .with_secret_key(SECRET_KEY)
                .verify()?
        )
    }
}

/// Web client posts this to create a new user.
#[derive(Serialize, Deserialize)]
struct RegisterInfo {
    email: String,
    password: String,
    display_name: String,
    real_name: String
}

/// Web client posts this to login.
#[derive(Serialize, Deserialize)]
struct LoginInfo {
    email: String,
    password: String
}

/// Information about the requested user
///
/// Meant to be returned from /api/me
// TODO: reorganize this whole file so intent is clearer
#[derive(Serialize, Deserialize)]
struct UserInfo {
    email: String,
    display_name: String,
    real_name: String
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            email: user.email,
            display_name: user.display_name,
            real_name: user.real_name
        }
    }
}

/// Route used to create a new user
// TODO: right now my error type does not implement responder so returning an error
// here returns a 500 to the client and logs the error to the console
#[post("/signup", format = "json", data = "<reg_info>")]
fn signup(mut cookies: Cookies, reg_info: Json<RegisterInfo>, db: State<DbConn>) -> Result<status::Created<()>, Error> {
    let conn = db.lock().unwrap();
    User::create_new(&conn, &reg_info)?;
    // we need to load the user right back from the db so we have the correct user_id
    let user = User::load_email(&conn, &reg_info.email)?;

    cookies.add_private(Cookie::new("user_id", user.user_id.to_string()));
    Ok(status::Created("".to_string(), None))
}

#[post("/login", format = "json", data = "<login_info>")]
fn login(mut cookies: Cookies, login_info: Json<LoginInfo>, db: State<DbConn>) -> Result<status::Accepted<()>, Error> {
    let conn = db.lock().unwrap();
    let user = User::load_email(&conn, &login_info.email)?;

    if user.auth(&login_info)? {
        cookies.add_private(Cookie::new("user_id", user.user_id.to_string()));
        Ok(status::Accepted(None))
    } else {
        Err(Error::LoginFailed)
    }
}

#[post("/logout")]
fn logout(mut cookies: Cookies) -> Status {
    cookies.remove_private(Cookie::named("user_id"));
    Status::Ok
}

#[post("/me")]
fn me_authed(user: User) -> Json<UserInfo> {
    Json(user.into())
}

#[post("/me", rank = 2)]
fn me() -> status::Custom<()> {
    status::Custom(Status::Unauthorized, ())
}

/// A "catch-all" to redirect path requests to the index since we are building a SPA
#[catch(404)]
fn not_found() -> NamedFile {
    NamedFile::open(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/svelte-app/public/index.html"))).unwrap()
}

/// Performs any necessary database setup upon application start.
///
/// Can be called multiple times without issue.
fn init_database(conn: &Connection) -> Result<(), Error> {
    User::create_table(conn)
}

fn rocket() -> Result<rocket::Rocket, Error> {
    // TODO: more configurable persistence?
    let conn = Connection::open(concat!(env!("CARGO_MANIFEST_DIR"), "/db.db3"))?;
    init_database(&conn)?;

    Ok(
        rocket::ignite()
            .manage(Mutex::new(conn))
            // TODO: bundle static files into binary for easy deploy?
            .mount("/", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/svelte-app/public")))
            .mount("/api", routes![signup, login, logout, me, me_authed])
            .register(catchers![not_found])
    )
}

fn main() -> Result<(), Error> {
    // rocket pretty prints the error when it drops if one occurs
    let _ = rocket()?.launch();
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_user_database() -> Result<(), Error> {
        let conn = Connection::open_in_memory()?;
        User::create_table(&conn)?;

        let me = User {
            user_id: 1,
            hash: "kalnfdanf".to_string(),
            email: "some_email@gmail.com".to_string(),
            created_at: Utc::now().naive_utc(),
            display_name: "display_name".to_string(),
            real_name: "real_name".to_string()
        };
        conn.execute(
            "INSERT INTO user (user_id, hash, email, created_at, display_name, real_name)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![me.user_id, me.hash, me.email, me.created_at, me.display_name, me.real_name],
        )?;

        let user = User::load_id(&conn, 1)?;
        assert_eq!(&user, &me);

        let user = User::load_email(&conn, &me.email)?;
        assert_eq!(&user, &me);

        Ok(())
    }

    #[test]
    fn create_user_from_info() -> Result<(), Error> {
        let conn = Connection::open_in_memory()?;
        User::create_table(&conn)?;
        let password = "myAmazingPassw0rd!".to_string();

        let rinfo = RegisterInfo {
            email: "some_email@gmail.com".to_string(),
            password: password.clone(),
            display_name: "Cldfire".to_string(),
            real_name: "Some Person".to_string()
        };

        User::create_new(&conn, &rinfo)?;
        let user = User::load_id(&conn, 1)?;
        assert_eq!(user.auth(&LoginInfo { email: "".to_string(), password })?, true);

        Ok(())
    }

    #[test]
    fn test_load_invalid_id() -> Result<(), Error> {
        let conn = Connection::open_in_memory()?;
        User::create_table(&conn)?;

        let res = User::load_id(&conn, 1);

        // error should be "QueryReturnedNoRows"
        assert_eq!(res, Err(Error::DatabaseErr(rusqlite::Error::QueryReturnedNoRows)));
        Ok(())
    }
}
