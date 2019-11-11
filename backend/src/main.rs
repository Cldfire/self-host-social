use argonautica::{Hasher, Verifier};
use chrono::{NaiveDateTime, Utc};
use rusqlite::{params, Connection, Result as SqliteResult};
use serde_derive::{Deserialize, Serialize};
use warp::{self, path, Filter};

use std::sync::{Arc, Mutex};

type DbState = Arc<Mutex<Connection>>;

const SECRET_KEY: &str = "TODO: HANDLE SECRET KEY";

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
    fn create_new(conn: &Connection, rinfo: &RegisterInfo) -> SqliteResult<()> {
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
    // TODO: what happens if invalid id is passed in?
    fn load_from(conn: &Connection, user_id: u64) -> SqliteResult<Self> {
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

fn main() -> SqliteResult<()> {
    let conn = Connection::open_in_memory()?;
    User::create_table(&conn)?;

    let db_state = Arc::new(Mutex::new(conn));

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    // let register = path!("register")
    //     .and(warp::body::content_length_limit(1024 * 8))
    //     .and(warp::body::json())
    //     .map(|info: RegisterInfo| {
    //         // TODO: handle erroring if user tries to sign up with an email or
    //         // real name that already exists

    //         // TODO: all usage of the database is blocking, this is in an async
    //         // context so that is bad
    //         //
    //         // not a huge deal since this webserver will never be hit by many requests
    //         // at the same time but should still fix

    //         // TODO: password hashing is also blocking and should be handled better

    //         let state_clone = db_state.clone();
    //         let conn = state_clone.lock().unwrap();
    //     });

    // warp::serve(register)
    //     .run(([127, 0, 0, 1], 1111));

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_user_database() -> SqliteResult<()> {
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
    fn create_user_from_info() -> SqliteResult<()> {
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
}
