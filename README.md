# self-host-social

A project that aims to implement a small, self-contained, self-hostable social network in the style of Instagram and Facebook for use by small groups of 20-30 people.

## Frontend

The frontend is in `svelte-app` and is written in JS with the Svelte 3 framework. Run `npm install` and then `npm build`.

I still need to set up live reloading w/ the Rust backend, so as of right now you'll have to re-run `npm build` after every change.

## Backend

The backend is in `backend` and is written in Rust. From the repo root:

* `cargo run` to run the webserver
* `cargo test` to run tests
