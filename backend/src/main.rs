use chrono::{NaiveDateTime, Utc};
use rusqlite::{params, Connection, Result as SqliteResult};

#[derive(Debug, PartialEq)]
struct User {
    /// The user's unique, numeric ID.
    ///
    /// This number starts at 0 (the first user) and is incremented for each
    /// user that signs up.
    user_id: u32,
    hash: Vec<u8>,
    salt: String,
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
    fn create_table(conn: &Connection) -> SqliteResult<()> {
        Ok(conn.execute(
            "CREATE TABLE user (
                    user_id                 INTEGER PRIMARY KEY,
                    hash                    BLOB NOT NULL,
                    salt                    TEXT NOT NULL,
                    email                   TEXT NOT NULL,
                    created_at              TEXT NOT NULL,
                    display_name            TEXT NOT NULL,
                    real_name               TEXT NOT NULL
                    )",
            params![],
        ).map(|_| ())?)
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_user_database() -> SqliteResult<()> {
        let conn = Connection::open_in_memory()?;
        User::create_table(&conn)?;

        let me = User {
            user_id: 0,
            hash: vec![0, 0, 1, 2],
            salt: "some_salt".to_string(),
            email: "some_email@gmail.com".to_string(),
            created_at: Utc::now().naive_utc(),
            display_name: "display_name".to_string(),
            real_name: "real_name".to_string()
        };
        conn.execute(
            "INSERT INTO user (user_id, hash, salt, email, created_at, display_name, real_name)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![me.user_id, me.hash, me.salt, me.email, me.created_at, me.display_name, me.real_name],
        )?;

        let mut stmt = conn.prepare("SELECT user_id, hash, salt, email, created_at, display_name, real_name FROM user")?;
        let users = stmt.query_map(params![], |row| {
            Ok(User {
                user_id: row.get(0)?,
                hash: row.get(1)?,
                salt: row.get(2)?,
                email: row.get(3)?,
                created_at: row.get(4)?,
                display_name: row.get(5)?,
                real_name: row.get(6)?
            })
        })?.collect::<Vec<_>>();

        assert!(users.len() == 1);
        assert_eq!(users[0].as_ref().unwrap(), &me);

        Ok(())
    }
}
