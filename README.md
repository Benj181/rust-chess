# rust-chess

A chess engine in Rust, built on bitboards, growing into a multiplayer chess
website written in Rust from front to back with Leptos and Axum.

The engine has a full board representation with FEN parsing and legal move
generation for every piece, including castling, en passant and promotions.
Perft results match the published numbers for the standard test positions.
On top of that, a `Game` type plays moves, keeps the history and detects
checkmate, stalemate and the automatic draws.

Next come the web app (a board you can play on in the browser), online games
between two players over WebSockets, and a computer opponent.

## Usage

```rust
use engine::{board::Board, movegen::generate_legal, moves::MoveList};

let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
let mut moves = MoveList::new();
generate_legal(&board, &mut moves);
assert_eq!(moves.len(), 20);
```

## Running the web app

The web app uses [Leptos](https://leptos.dev) and [Axum](https://github.com/tokio-rs/axum),
built with [cargo-leptos](https://github.com/leptos-rs/cargo-leptos). Install the
tools once:

```sh
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
```

Then start the dev server:

```sh
cargo leptos watch
```

Open [http://127.0.0.1:3000](http://127.0.0.1:3000). The server rebuilds on
every save and the page reloads by itself. Stop it with `Ctrl+C`.

`cargo leptos build --release` makes an optimised build in `target/`.

## Documentation

The API docs are online at
**[benj181.github.io/rust-chess](https://benj181.github.io/rust-chess/engine/)**,
rebuilt on every push to `main`.

To build them locally and open them in your browser:

```sh
cargo doc --open
```

## Tests

```sh
cargo test
```

This runs the perft suite in `engine/tests/perft.rs` and the examples in
the docs.
