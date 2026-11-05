# rust-chess

A chess engine in Rust, growing into an original, styled multiplayer chess
website (like Lichess or chess.com) built with **Leptos** and **Axum**. This is
a learning project. The chess engine is mostly done; the main goal now is to
**learn basic web development with Rust**.

## How to work with me (the user)

- **Teach, don't do.** By default, give small step-by-step hints and let me
  write the code. Only write code when I ask ("can you write it", "finish it").
  When you review my code, point out bugs and lints, and explain why.
- **One small step at a time.** Long walls of text are hard to follow. Give a
  little, then wait for me.
- **I'm new to Leptos and mostly new to web development.** Go very slowly in
  the web milestones: one concept at a time, explain the concept before using
  it, and connect it to Rust I already know.
- **No code comments.** Doc comments (`///`, `//!`) on the public engine API
  are wanted; ordinary `//` comments are not.
- **I dislike writing tests.** The only kept test is the perft integration test
  (`engine/tests/perft.rs`). Verify behavior in a scratch copy of the crate or
  a throwaway example, not in project files.
- **Git:** commit straight to `main`, never on a feature branch. Don't push
  unless I ask. End commit messages with the `Co-Authored-By` line.
- I edit in Neovim (with neo-tree) while you work, so files can change on disk
  under you. Re-read before editing, and don't overwrite my in-progress code
  without saying so.

## Workspace layout

| Crate | Purpose |
|---|---|
| `engine/` | Chess logic: bitboards, FEN, move generation, `Game`. No web deps. |
| `app/` | Leptos components and routes, shared by server and browser (features `ssr` / `hydrate`). |
| `frontend/` | WASM entry point: `hydrate()` mounts `App` in the browser. |
| `server/` | Axum binary: serves SSR pages, static files and (later) WebSockets. |

Also: `style/main.css` (CSS variables `--bg`, `--fg`, `--accent`),
`public/favicon.svg`, and the Leptos config in `[[workspace.metadata.leptos]]`
in the root `Cargo.toml`.

**Stack:** Leptos 0.8 (SSR + hydration), leptos_axum, axum 0.8 (`ws`
feature), tokio, cargo-leptos 0.3.x, target `wasm32-unknown-unknown`.

**Commands:**
- `cargo leptos watch`: dev server at http://127.0.0.1:3000 with auto-reload.
- `cargo leptos build`: full build.
- `cargo clippy -p engine --all-targets`: must be clean.
- `cargo test -p engine`: perft plus doctests.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p engine`: the docs CI check.

## Engine conventions (strict lints)

The workspace denies clippy `pedantic` and `nursery`, plus `unwrap_used`,
`expect_used`, `indexing_slicing`, `arithmetic_side_effects`, `panic`,
`unreachable`, `todo`, `unimplemented`, `string_slice`, `as_conversions`,
`panic_in_result_fn`, `exit` and `unchecked_time_subtraction`. `clippy.toml`
allows unwrap, expect, indexing and panic in tests. The web crates
(`app`, `frontend`, `server`) use default lints.

Idioms used throughout:
- `?` on `Option`, `is_some_and`, `map_or`, `checked_*` / `saturating_*`,
  `u8::try_from(..).ok()?`, `.get(..)` instead of indexing or slicing.
- `(file ^ rank) & 1` instead of `%` for square color.
- `const fn` wherever clippy allows, and `#[must_use]` on getters (not on
  functions returning iterators, which triggers `double_must_use`).
- Nested `use crate::{...}` imports, let-chains (edition 2024), fn pointers.
- Public functions returning `Result` need a `# Errors` doc section.

## Engine API (done)

- `square::Square`: `from_algebraic`, `from_file_rank`, `file`, `rank`, `Display` (`e4`).
- `bitboard::Bitboard`: set ops, `is_empty`, iterates its squares.
- `piece::{Color, PieceKind, Piece}`: `from_char` / `to_char`.
- `moves::{Move, MoveKind, MoveList}`: `Move` is a `Copy` `u16` with
  `from`/`to`/`kind`. `MoveList` is a fixed array of 256 with `iter`,
  `contains`, `len`.
- `board::Board`: `from_fen`, `to_fen`, `make_move` (copy-make), `piece_at`,
  `in_check`, `king_square`, `halfmove_clock`, `same_position`,
  `insufficient_material`, `Display`.
- `movegen`: `generate_legal`, `generate_pseudo_legal`, `perft`. Verified by
  perft on 5 standard positions.
- `game::Game`: `from_fen`, `board`, `moves`, `result`, `legal_moves` (empty
  once over), `play(Move)`, `play_uci("e2e4" / "e7e8q")`, `undo`,
  `resign(Color)`, `agree_draw`. It keeps a `history: Vec<Board>` and detects
  checkmate, stalemate, the 50-move rule, threefold repetition and
  insufficient material automatically after every move.
- `GameResult { Win { winner, reason: WinReason }, Draw(DrawReason) }` and
  `MoveError { GameOver, IllegalMove, InvalidNotation }`.

Known simplification: threefold compares the en passant square even when no
capture is possible (Lichess only counts it when it is). Fine for now.

Docs: https://benj181.github.io/rust-chess/engine/ (built by
`.github/workflows/docs.yml` on every push to `main`).

## Roadmap

1. **Game logic** (`engine/src/game.rs`): done. Losing on time comes with the
   server clocks in milestone 3.
2. **Local board UI in Leptos**: in progress, see below.
3. **Multiplayer**: WebSocket game rooms in Axum. The server is authoritative
   and validates every move with `Game::play_uci`. Clocks and timeouts.
4. **AI**: evaluation plus alpha-beta search, with Zobrist hashing.
5. **Polish**: original styling, my own SVG pieces, sounds, animations.
6. Optional: persistence and deployment.

## Current status: start of milestone 2

Next session: **start step 1, going very slowly, one Leptos concept at a
time.** `app/src/lib.rs` still has a placeholder `HomePage` that shows the
legal move count of the start position.

Milestone 2 plan:
1. **Structure:** split `app/src/lib.rs` into modules (for example
   `pages/play.rs` and `components/board.rs`). Point the `""` route at the play
   page. Add `Game::new()` or `impl Default for Game` with the start position.
2. **State:** put the game in a signal, `RwSignal::new(Game::new())`. Read it
   with `game.with(..)` and change it with `game.update(..)`. Derive
   everything else from it.
3. **Board:** an 8×8 CSS grid looping rank 7 down to 0, with `light`/`dark`
   classes and Unicode pieces for now. Each square reads the piece through a
   `move ||` closure, so it updates by itself.
4. **Click to move:** a `selected: RwSignal<Option<Square>>`. Find the legal
   move by from and to (like `play_uci`) and auto-queen promotions for now.
5. **Highlights:** selected square, legal targets (dots or capture rings),
   last move, king in check.
6. **Side panel:** a status line from `result()`; a move list (needs
   `Display` for `Move` printing UCI; SAN is a later engine step); New game,
   Undo and Flip buttons.
7. **Optional polish:** promotion picker, drag and drop.
