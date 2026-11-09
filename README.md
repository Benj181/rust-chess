# rust-chess

A chess engine in Rust, built on bitboards, with a chess website on top of it
written in Rust from front to back with [Leptos](https://leptos.dev) and
[Axum](https://github.com/tokio-rs/axum).

**Play it: [benj181.github.io/rust-chess](https://benj181.github.io/rust-chess/)**

The engine has a full board representation with FEN parsing and legal move
generation for every piece, including castling, en passant and promotions.
Perft results match the published numbers for the standard test positions.
On top of that, a `Game` type plays moves, keeps the history, writes moves in
standard algebraic notation and detects checkmate, stalemate and the
automatic draws.

The web app is a board you can play on in the browser: click or drag pieces,
pick a promotion piece, step through the game with the move list, the
buttons or the arrow keys, flip the board, see captured material, take moves
back, offer a draw or resign.

Next comes a computer opponent that runs in the browser, and later online
games between two players over WebSockets.

## Two versions

The same UI is built in two ways:

- **Standalone** runs entirely in the browser (client-side rendering). It is
  just static files, so it is hosted for free on GitHub Pages. This is the
  version for playing against the computer.
- **Server** is an Axum server that renders the page and then hands it to the
  browser (server-side rendering with hydration). It is the version for
  online games between two players.

## Project structure

| Crate | What it is |
|---|---|
| `engine/` | The chess rules: bitboards, FEN, move generation, `Game`. No web dependencies. |
| `app/` | The Leptos UI (board, move list, controls), shared by both versions. |
| `standalone/` | Entry point of the standalone version: mounts the app in an empty page. Built with [Trunk](https://trunkrs.dev). |
| `server/` | The Axum server of the server version. |
| `frontend/` | The browser half of the server version: hydrates the page the server sent. |

Also `style/main.css` (all styling, shared by both versions) and `public/`
(static files such as the favicon).

## Usage

```rust
use engine::{board::Board, movegen::generate_legal, moves::MoveList};

let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
let mut moves = MoveList::new();
generate_legal(&board, &mut moves);
assert_eq!(moves.len(), 20);
```

## Running the web app

Install the WASM target once:

```sh
rustup target add wasm32-unknown-unknown
```

### Standalone version

Install [Trunk](https://trunkrs.dev) once:

```sh
cargo install trunk --locked
```

Then start it from the `standalone` folder:

```sh
cd standalone
trunk serve
```

Open [http://127.0.0.1:8080/rust-chess/](http://127.0.0.1:8080/rust-chess/).
It rebuilds on every save. `trunk build --release` writes an optimised build
to `standalone/dist/`.

### Server version

Install [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) once:

```sh
cargo install cargo-leptos --locked
```

Then start the dev server from the repository root:

```sh
cargo leptos watch
```

Open [http://127.0.0.1:3000](http://127.0.0.1:3000). The server rebuilds on
every save and the page reloads by itself. Stop it with `Ctrl+C`.
`cargo leptos build --release` makes an optimised build in `target/`.

## Documentation

The engine's API docs are online at
**[benj181.github.io/rust-chess/engine](https://benj181.github.io/rust-chess/engine/)**.

To build them locally and open them in your browser:

```sh
cargo doc --open -p engine
```

## Deployment

Every push to `main` runs `.github/workflows/pages.yml`, which builds the
standalone version with Trunk and the engine docs, and publishes both to
GitHub Pages: the game at `/rust-chess/` and the docs at
`/rust-chess/engine/`.

## Tests

```sh
cargo test -p engine
```

This runs the perft suite in `engine/tests/perft.rs` and the examples in
the docs.
