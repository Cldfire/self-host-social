# self-host-social

A project that aims to implement a small, self-contained, self-hostable social network in the style of Instagram and Facebook for use by small groups of 20-30 people.

## Frontend

The frontend is in `svelte-app` and is written in JS with the Svelte 3 framework. Run `npm install` and then `npm run watch`.

(Make sure you also start the backend seperately. Live reload should be working.)

## Backend

The backend is in `backend` and is written in Rust. From the repo root:

* `cargo run` to run the webserver
* `cargo test` to run tests

## Deploying

Right now the deploy process is as follows:

* Run `npm run build` from the `svelte-app` folder
* Build a deployable binary with `cargo run --release --features deployable`
    * Note: if you are building for a server with an older `libc` version installed like I am, you'll want to add a `--target x86_64-unknown-linux-musl` to statically link a newer `libc` version
    * Make sure you have `musl` stuff installed first. On Arch, `sudo pacman -S musl`
* Use `scp` to upload this binary to a server somewhere
    * Example: `scp target/x86_64-unknown-linux-musl/release/backend your_username@remotehost.edu:/some/remote/directory/backend`
* Use `scp` to upload the contents of the `svelte-app/public` folder to a `static` folder next to the binary
    * Example: `scp -r svelte-app/public your_username@remotehost.edu:/some/remote/directory/static`
* Create a `Rocket.toml` file in the same directory as the backend binary with the following contents:

```toml
[production]
address = "localhost"
port = 8000
log = "critical"
secret_key = "..."
argon_secret_key = "..."
```

where `secret_key` and `argon_secret_key` are generated using something like `openssl rand -base64 32`.

* Set up a reverse proxy of your choice (nginx?) to handle TLS and proxy requests to the backend
    * You could also use Rocket's TLS support and skip the reverse proxy, but according to the author it is not ready for production
* Enjoy!
