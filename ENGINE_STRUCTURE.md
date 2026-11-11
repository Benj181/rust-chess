# Engine structure (planned)

Planned restructure of `engine/src/`, which is currently 11 flat files.
The goal: each module does one job, and dependencies only point downwards.

## Tree

```
engine/src/
├── lib.rs              mod lines + pub use re-exports
│
├── types.rs            mod lines + pub use
├── types/
│   ├── square.rs       Square
│   ├── piece.rs        Color, PieceKind, Piece
│   ├── bitboard.rs     Bitboard
│   ├── moves.rs        Move, MoveKind, MoveList
│   └── castling.rs     CastlingRights, Castle
│
├── board.rs            Board: fields, queries, make_move, Display
├── board/
│   ├── attacks.rs      pub(crate): knight/king/pawn/sliding attacks
│   └── direction.rs    pub(crate): Direction
│
├── movegen.rs          generate_legal, generate_pseudo_legal, perft
│
├── notation.rs         mod lines
├── notation/
│   ├── fen.rs          Board::from_fen / to_fen / start, START_FEN,
│   │                   CastlingRights' FEN parsing and formatting
│   ├── san.rs          Board::san
│   └── uci.rs          parse_uci (moved out of game.rs)
│
├── game.rs             Game
└── game/
    └── result.rs       GameResult, WinReason, DrawReason, MoveError
```

## Layers

```
      game          Game, results, rules over a whole game
       │
    notation        text formats: FEN, SAN, UCI
       │
    movegen         which moves are legal
       │
     board          one position, attacks, make_move
       │
     types          Square, Piece, Bitboard, Move, CastlingRights
```

Each layer only uses the layers below it. No cycles.

| Module | Uses |
|---|---|
| `types` | nothing outside `types` |
| `board` | `types` |
| `movegen` | `board` (including `board::attacks`, `board::direction`), `types` |
| `notation` | `movegen` (SAN needs legal moves for disambiguation and `+`/`#`), `board`, `types` |
| `game` | `notation` (`san_moves`, `from_fen`, `play_uci`), `movegen`, `board`, `types` |

Later the AI (milestone 4) gets its own `search.rs` + `search/` (eval,
Zobrist, alpha-beta) above `movegen`.

## Rules of thumb

- **Small text helpers stay on their type.** `Square::from_algebraic`,
  `Piece::from_char` / `to_char` and the `Display` impls (`e4`, `white`,
  UCI for `Move`) are a type's own spelling, not a notation format. Moving
  them into `notation` would make `types` depend on `notation`.
- **Submodules use the `name.rs` + `name/` style** (not `mod.rs`). `board.rs`
  declares `pub(crate) mod attacks;` and Rust looks in `board/attacks.rs`.
- **`impl` blocks can live anywhere in the crate.** `notation/fen.rs` and
  `notation/san.rs` each have their own `impl Board { ... }`.
- **Visibility:**

  | | visible to |
  |---|---|
  | `mod x;` | the parent and its children |
  | `pub(crate) mod x;` | the whole engine, not the app |
  | `pub mod x;` | everyone |

## Decisions

1. `Board::start()` and `START_FEN` move into `notation/fen.rs`, so `board`
   doesn't depend on `notation`.
2. `Board`'s fields become `pub(crate)`, so `notation/fen.rs` can build a
   board. The app still can't touch them. (Alternative: keep them private
   and add `pub(crate)` setters.)

## Public API

`lib.rs` re-exports the main types, so users import from the root:

```rust
pub use board::Board;
pub use game::{Game, GameResult, MoveError};
pub use types::{Color, Move, Piece, PieceKind, Square};
```

`types.rs` re-exports its contents, so engine code writes
`use crate::types::{Bitboard, Color, Square};`.

Old paths like `engine::piece::Color` disappear. The app's three `use engine`
blocks and the doctest in `lib.rs` switch to `engine::Color` and so on.

## Order of work

Run `cargo test -p engine` and `cargo clippy -p engine --all-targets` after
each step.

1. `types`: create `types.rs`, move the five files into `types/`, fix the
   `crate::` imports.
2. Root re-exports in `lib.rs`; update the app's imports and the doctest.
3. `board/attacks.rs` and `board/direction.rs`.
4. `notation`: `fen.rs` (with `start` and `CastlingRights`' FEN), `san.rs`,
   `uci.rs`.
5. `game/result.rs`.
6. Update the workspace layout and engine API sections in `CLAUDE.md`.
