#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use rocket::outcome::Outcome;
use rocket::http::Status;
use rocket::response::{Content, NamedFile};
use rocket::response::status;
use rocket::request::{self, FromRequest, Request};
use rocket::http::{Cookie, Cookies, ContentType};
use rocket::Data;
use rocket::State;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

use argonautica::{Hasher, Verifier};
use chrono::{NaiveDateTime, Utc};
use rusqlite::{params, Connection};
use serde_derive::{Deserialize, Serialize};
use image::load_from_memory;

use identicon_rs::{Identicon, ImageType};

use std::sync::Mutex;
use std::path::Path;
use std::io::Read;

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
    RocketLaunchErr,
    /// Placeholder error returned when failure to read uploaded image data occurs
    ImageUploadFailed
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
    real_name: String,
    // The bytes of the PNG-encoded profile picture
    profile_pic: Vec<u8>
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
                    real_name               TEXT NOT NULL,
                    profile_pic             BLOB NOT NULL
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

        let identicon = Identicon::new_default(&(rinfo.display_name.clone() + &rinfo.email + &rinfo.real_name));
        let data = identicon.export_file_data(ImageType::PNG);

        Ok(conn.execute(
            "INSERT INTO user (hash, email, created_at, display_name, real_name, profile_pic)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![hash, rinfo.email, created_at, rinfo.display_name, rinfo.real_name, data],
        ).map(|_| ())?)
    }

    /// Loads the user specified by the given id from the database.
    fn load_id(conn: &Connection, user_id: u32) -> Result<Self, Error> {
        Ok(conn.query_row(
            &("SELECT user_id, hash, email, created_at, display_name, real_name, profile_pic FROM user WHERE user_id='".to_string() + &user_id.to_string() + "'"),
            params![],
            |row| {
                Ok(User {
                    user_id: row.get(0)?,
                    hash: row.get(1)?,
                    email: row.get(2)?,
                    created_at: row.get(3)?,
                    display_name: row.get(4)?,
                    real_name: row.get(5)?,
                    profile_pic: row.get(6)?
                })
            }
        )?)
    }

    /// Loads the user specified by the given email from the database
    fn load_email(conn: &Connection, email: &str) -> Result<Self, Error> {
        Ok(conn.query_row(
            &("SELECT user_id, hash, email, created_at, display_name, real_name, profile_pic FROM user WHERE email='".to_string() + email + "'"),
            params![],
            |row| {
                Ok(User {
                    user_id: row.get(0)?,
                    hash: row.get(1)?,
                    email: row.get(2)?,
                    created_at: row.get(3)?,
                    display_name: row.get(4)?,
                    real_name: row.get(5)?,
                    profile_pic: row.get(6)?
                })
            }
        )?)
    }

    /// Returns a buffer with the profile pic for the user specified by the given id
    // TODO: right now images are loaded into memory in their entirety from the database
    //
    // this might not be an issue as this server will never deal with large images buuuut
    // it might be nice to fix regardless
    //
    // might be better to store images outside the database in the filesystem to avoid
    // fighting with the sqlite API
    fn get_profile_pic(conn: &Connection, user_id: u32) -> Result<Vec<u8>, Error> {
        Ok(conn.query_row(
            &("SELECT profile_pic FROM user WHERE user_id='".to_string() + &user_id.to_string() + "'"),
            params![],
            |row| {
                Ok(row.get(0)?)
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

/// Representation of a post in the database
#[derive(Debug, PartialEq)]
struct Post {
    id: u32,
    body: String,
    created_at: NaiveDateTime,
    // TODO: how to differentiate between png / jpeg?
    image: Option<Vec<u8>>,
    /// The user that made this post
    user_id: u32
}

impl Post {
    /// Creates a table in the given database for storing this struct.
    ///
    /// The table will only be created if it does not already exist.
    fn create_table(conn: &Connection) -> Result<(), Error> {
        Ok(conn.execute(
            "CREATE TABLE if not exists post (
                    id                      INTEGER PRIMARY KEY,
                    body                    TEXT NOT NULL,
                    created_at              TEXT NOT NULL,
                    image                   BLOB,
                    user_id                 INTEGER
                    )",
            params![],
        ).map(|_| ())?)
    }

    /// Inserts a new post into the database based on the given post info and user id.
    // TODO: remove that requirement when Rocket gets multipart form support
    fn create_new(conn: &Connection, pinfo: &PostInfo, user_id: u32) -> Result<(), Error> {
        let created_at = Utc::now().naive_utc();

        Ok(conn.execute(
            "INSERT INTO post (body, created_at, user_id)
                    VALUES (?1, ?2, ?3)",
            params![pinfo.body, created_at, user_id],
        ).map(|_| ())?)
    }

    /// Set the image of the post specified by the given id to the given image data.
    // TODO: this should support streaming so the entire image doesn't have to be
    // loaded in memory
    fn set_image(conn: &Connection, post_id: u32, data: &[u8]) -> Result<(), Error> {
        Ok(conn.execute(
            "UPDATE post SET image=?1 WHERE id=?2",
            params![data, post_id],
        ).map(|_| ())?)
    }

    /// Loads the post specified by the given id from the database.
    // TODO: pretty sure I can get rid of the string concat and use the params![]
    // instead in all of these
    fn load_id(conn: &Connection, post_id: u32) -> Result<Self, Error> {
        Ok(conn.query_row(
            &("SELECT id, body, created_at, image, user_id FROM post WHERE id='".to_string() + &post_id.to_string() + "'"),
            params![],
            |row| {
                Ok(Post {
                    id: row.get(0)?,
                    body: row.get(1)?,
                    created_at: row.get(2)?,
                    image: row.get(3)?,
                    user_id: row.get(4)?
                })
            }
        )?)
    }
}

/// Web client posts this to create a new user
#[derive(Serialize, Deserialize)]
struct RegisterInfo {
    email: String,
    password: String,
    display_name: String,
    real_name: String
}

/// Web client posts this to create a new post
#[derive(Serialize, Deserialize)]
struct PostInfo {
    body: String
}

/// Web client receives this after creating a post
#[derive(Serialize, Deserialize)]
struct PostCreationResponse {
    post_id: u32
}

/// Web client posts this to login.
#[derive(Serialize, Deserialize)]
struct LoginInfo {
    email: String,
    password: String
}

/// Information about the requested user
// TODO: reorganize this whole file so intent is clearer
#[derive(Serialize, Deserialize)]
struct UserInfo {
    user_id: u32,
    display_name: String,
    real_name: String
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            user_id: user.user_id,
            display_name: user.display_name,
            real_name: user.real_name
        }
    }
}

/// Returns information about the requested user id
#[get("/user-info/<req_user_id>")]
fn user_info(_user: User, db: State<DbConn>, req_user_id: u32) -> Result<Json<UserInfo>, Error> {
    let conn = db.lock().unwrap();
    let user = User::load_id(&conn, req_user_id)?;

    Ok(Json(user.into()))
}

/// Returns the profile picture for the requested user id
#[get("/profile-pic/<req_user_id>")]
fn profile_pic(_user: User, db: State<DbConn>, req_user_id: u32) -> Result<Content<Vec<u8>>, Error> {
    let conn = db.lock().unwrap();
    Ok(Content(ContentType::PNG, User::get_profile_pic(&conn, req_user_id)?))
}

/// Creates a post for whatever user makes the request using the provided post
/// information.
///
/// Returns the post's id upon successful creation.
#[post("/create-post", format = "json", data = "<post_info>")]
fn create_post(user: User, db: State<DbConn>, post_info: Json<PostInfo>) -> Result<status::Created<Json<PostCreationResponse>>, Error> {
    let conn = db.lock().unwrap();
    Post::create_new(&conn, &post_info, user.user_id)?;

    Ok(status::Created("".to_string(), Some(Json(PostCreationResponse { post_id: conn.last_insert_rowid() as u32 }))))
}

/// Set the image for the post with the given id to the provided image data.
// TODO: all of these routes should be put into a tree structure instead of
// being flat
//
// for example /api/post/<id>/image/set instead of /api/set-post-image/<id>
#[post("/set-post-image/<post_id>", format = "plain", data = "<data>")]
fn set_post_image(_user: User, db: State<DbConn>, data: Data, post_id: u32) -> Result<(), Error> {
    let conn = db.lock().unwrap();
    let mut data_buf = vec![];
    let mut jpeg_image_data = vec![];

    // 10 MB limit
    // TODO: more precise error handling throughout
    // TODO: reuse a single buffer rather than writing image data to a separate one.
    // too lazy to dirty the code doing it right now :D
    data.open().take(10485760).read_to_end(&mut data_buf).map_err(|_| Error::ImageUploadFailed)?;
    load_from_memory(&data_buf)
        .map_err(|_| Error::ImageUploadFailed)?
        .resize(1000, 1000, image::FilterType::CatmullRom)
        .write_to(&mut jpeg_image_data, image::ImageOutputFormat::JPEG(90))
        .map_err(|_| Error::ImageUploadFailed)?;

    Post::set_image(&conn, post_id, &jpeg_image_data).map(|_| ())
}

/// Route used to create a new user
// TODO: right now my error type does not implement responder so returning an error
// here returns a 500 to the client and logs the error to the console
#[post("/signup", format = "json", data = "<reg_info>")]
fn signup(mut cookies: Cookies, reg_info: Json<RegisterInfo>, db: State<DbConn>) -> Result<status::Created<Json<UserInfo>>, Error> {
    let conn = db.lock().unwrap();
    User::create_new(&conn, &reg_info)?;

    let user_id = conn.last_insert_rowid() as u32;
    let user_info = UserInfo {
        user_id: user_id,
        // TODO: these clones are unnecessary
        display_name: reg_info.display_name.clone(),
        real_name: reg_info.real_name.clone()
    };

    // TODO: set the secure flag on this cookie when not in dev mode
    cookies.add_private(Cookie::new("user_id", user_id.to_string()));
    Ok(status::Created("".to_string(), Some(Json(user_info))))
}

// TODO: right now logging in will quickly fail if a user enters an email that doesn't
// exist, making it trivial to probe for whether or not an account exists on the server
//
// should fix that
#[post("/login", format = "json", data = "<login_info>")]
fn login(mut cookies: Cookies, login_info: Json<LoginInfo>, db: State<DbConn>) -> Result<status::Accepted<Json<UserInfo>>, Error> {
    let conn = db.lock().unwrap();
    let user = User::load_email(&conn, &login_info.email)?;

    if user.auth(&login_info)? {
        cookies.add_private(Cookie::new("user_id", user.user_id.to_string()));
        Ok(status::Accepted(Some(Json(user.into()))))
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
// TODO: perhaps we should still return a 404 for anything that's not a path?
#[catch(404)]
fn not_found() -> NamedFile {
    NamedFile::open(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/svelte-app/public/index.html"))).unwrap()
}

/// Performs any necessary database setup upon application start.
///
/// Can be called multiple times without issue.
fn init_database(conn: &Connection) -> Result<(), Error> {
    User::create_table(conn)?;
    Post::create_table(conn)
}

/// Create a Rocket instance managing the given database connection
fn rocket(conn: Connection) -> Result<rocket::Rocket, Error> {
    init_database(&conn)?;

    Ok(
        rocket::ignite()
            .manage(Mutex::new(conn))
            // TODO: bundle static files into binary for easy deploy?
            .mount("/", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/svelte-app/public")))
            .mount("/api", routes![
                signup,
                login,
                logout,
                me,
                me_authed,
                profile_pic,
                user_info,
                create_post,
                set_post_image
            ])
            .register(catchers![not_found])
    )
}

fn main() -> Result<(), Error> {
    // TODO: more configurable database persistence?
    // rocket pretty prints the error when it drops if one occurs
    let _ = rocket(
        Connection::open(concat!(env!("CARGO_MANIFEST_DIR"), "/db.db3"))?
    )?.launch();

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::Response;
    use rocket::local::Client;
    use std::fs::File;

    fn user_id_cookie(response: &Response) -> Option<Cookie<'static>> {
        let cookie = response.headers()
            .get("Set-Cookie")
            .filter(|v| v.starts_with("user_id"))
            .nth(0)
            .and_then(|val| Cookie::parse_encoded(val).ok());
    
        cookie.map(|c| c.into_owned())
    }

    fn login(client: &Client, email: String, password: String) -> Option<Cookie<'static>> {
        let login_info = LoginInfo {
            email,
            password
        };

        let response = client.post("/api/login")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&login_info).unwrap())
            .dispatch();

        user_id_cookie(&response)
    }

    #[test]
    fn test_user_database() -> Result<(), Error> {
        let conn = Connection::open_in_memory()?;
        init_database(&conn)?;

        let email = String::from("some_email@gmail.com");
        let display_name = String::from("display_name");
        let real_name = String::from("real_name");

        let identicon = Identicon::new_default(&(display_name.clone() + &email + &real_name));
        let data = identicon.export_file_data(ImageType::PNG);

        let me = User {
            user_id: 1,
            hash: "kalnfdanf".to_string(),
            email,
            created_at: Utc::now().naive_utc(),
            display_name,
            real_name,
            profile_pic: data
        };
        conn.execute(
            "INSERT INTO user (user_id, hash, email, created_at, display_name, real_name, profile_pic)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![me.user_id, me.hash, me.email, me.created_at, me.display_name, me.real_name, me.profile_pic],
        )?;

        let user = User::load_id(&conn, 1)?;
        assert_eq!(&user, &me);

        let user = User::load_email(&conn, &me.email)?;
        assert_eq!(&user, &me);

        let profile_pic = User::get_profile_pic(&conn, 1)?;
        assert_eq!(&me.profile_pic, &profile_pic);

        Ok(())
    }

    #[test]
    fn create_user_from_info() -> Result<(), Error> {
        let conn = Connection::open_in_memory()?;
        init_database(&conn)?;

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
        init_database(&conn)?;

        let res = User::load_id(&conn, 1);

        // error should be "QueryReturnedNoRows"
        assert_eq!(res, Err(Error::DatabaseErr(rusqlite::Error::QueryReturnedNoRows)));
        Ok(())
    }

    #[test]
    fn test_create_post_set_image() -> Result<(), Error> {
        let conn = Connection::open_in_memory()?;
        init_database(&conn)?;

        // TODO: write a function to create a test user to avoid the copy + paste
        let email = "some_email@gmail.com".to_string();
        let password = "myAmazingPassw0rd!".to_string();

        let rinfo = RegisterInfo {
            email: email.clone(),
            password: password.clone(),
            display_name: "Cldfire".to_string(),
            real_name: "Some Person".to_string()
        };

        User::create_new(&conn, &rinfo)?;

        let client = Client::new(rocket(conn)?).unwrap();
        let db = client.rocket().state::<DbConn>().unwrap();
        let login_cookie = login(&client, email, password).expect("logged in");

        let post_info = PostInfo {
            body: "This is the body of a post!".to_string()
        };

        let mut response = client
            .post("/api/create-post")
            .cookie(login_cookie.clone())
            .header(ContentType::JSON)
            .body(serde_json::to_string(&post_info).unwrap())
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        let response_json = response.body_string().unwrap();
        let created_post: PostCreationResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(created_post.post_id, 1);

        {
            // doing this in a new block so the mutex lock drops and allows the
            // second post to succeed
            let conn = db.lock().unwrap();
            let post = Post::load_id(&conn, 1)?;
            assert!(post.image.is_none());
            assert_eq!(&post.body, &post_info.body);
            assert_eq!(post.user_id, 1);
        }

        let mut file = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/svelte-app/public/favicon.png")).unwrap();
        let mut favicon_buf = vec![];
        file.read_to_end(&mut favicon_buf).unwrap();

        let response = client
            .post("/api/set-post-image/1")
            .cookie(login_cookie.clone())
            .header(ContentType::Plain)
            .body(&favicon_buf)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let conn = db.lock().unwrap();
        let post = Post::load_id(&conn, 1)?;

        assert!(post.image.is_some());
        // The image gets upscaled and re-encoded to JPEG by the server, so it
        // should be significantly larger now
        assert!(post.image.unwrap().len() > favicon_buf.len());

        Ok(())
    }
}
