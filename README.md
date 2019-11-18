# self-host-social

A project that aims to implement a small, self-contained, self-hostable social network in the style of Instagram and Facebook for use by small groups of 20-30 people.

## Frontend

The frontend is in `svelte-app` and is written in JS with the Svelte 3 framework. Run `npm install` and then `npm run watch`.

(Make sure you also start the backend seperately. Live reload should be working.)

## Backend

The backend is in `backend` and is written in Rust. From the repo root:

* `cargo run` to run the webserver
* `cargo test` to run tests
