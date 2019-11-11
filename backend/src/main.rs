use argonautica::{Hasher, Verifier};
use chrono::{NaiveDateTime, Utc};
use rusqlite::{params, Connection, Result as SqliteResult};
use serde_derive::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};

type DbState = Arc<Mutex<Connection>>;

const SECRET_KEY: &str = "TODO: HANDLE SECRET KEY";

/// The error type used throughout the binary
#[derive(Debug, PartialEq)]
enum Error {
    DatabaseErr(rusqlite::Error),
    /// Error returned when an attempt is made to create a new user with a real
    /// name or email address that already exists in the database.
    UserAlreadyExists
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::DatabaseErr(err)
    }
}

/// Representation of a user in the database.
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

impl User {
    /// Creates a table in the given database for storing this struct.
    ///
    /// The table will only be created if it does not already exist.
    fn create_table(conn: &Connection) -> SqliteResult<()> {
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
    fn load_from(conn: &Connection, user_id: u64) -> Result<Self, Error> {
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
}

/// Information to be received from the web client in order to register a new
/// user.
#[derive(Serialize, Deserialize)]
struct RegisterInfo {
    email: String,
    password: String,
    display_name: String,
    real_name: String
}

fn main() -> Result<(), Error> {
    let conn = Connection::open_in_memory()?;
    User::create_table(&conn)?;

    let db_state = Arc::new(Mutex::new(conn));

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

        let user = User::load_from(&conn, 1)?;

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
        let user = User::load_from(&conn, 1)?;

        let mut verifier = Verifier::default();
        let is_valid = verifier
            .with_hash(&user.hash)
            .with_password(password)
            .with_secret_key(SECRET_KEY)
            .verify()
            .unwrap();

        assert_eq!(is_valid, true);

        Ok(())
    }

    #[test]
    fn test_load_invalid_id() -> Result<(), Error> {
        let conn = Connection::open_in_memory()?;
        User::create_table(&conn)?;

        let res = User::load_from(&conn, 1);

        // error should be "QueryReturnedNoRows"
        assert_eq!(res, Err(Error::DatabaseErr(rusqlite::Error::QueryReturnedNoRows)));
        Ok(())
    }
}
