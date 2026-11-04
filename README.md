# rust-chess

A chess engine in Rust, built on bitboards.

So far it has a full board representation with FEN parsing and legal move
generation for every piece, including castling, en passant and promotions.
Perft results match the published numbers for the standard test positions.
Search and evaluation come next.

## Usage

```rust
use engine::{board::Board, movegen::generate_legal, moves::MoveList};

let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
let mut moves = MoveList::new();
generate_legal(&board, &mut moves);
assert_eq!(moves.len(), 20);
```

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
