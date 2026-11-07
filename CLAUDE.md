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

Also: `style/main.css` (all colours are design tokens on `:root`; no hex
values outside it),
`public/favicon.svg`, and the Leptos config in `[[workspace.metadata.leptos]]`
in the root `Cargo.toml`.

**Design reference:** `mono-chess-design.pdf` in the root ("Mono Chess"): a
dark, flat, monospace look (Fira Code only, no shadows or gradients, 2px
radius). Page 1 is the desktop board-and-moves screen; page 2 has the phone
layout, colour tokens, piece styling, geometry and behaviour (move history
navigation, keyboard keys, captures, clocks). Follow it for all UI work.
Parts of it need later work: SAN for the move list (engine), running clocks
(milestone 3) and SVG pieces (milestone 5).

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

## Current status: milestone 2, steps 1–5 and 6a–6c done; 6d next

The board is playable by clicking, with highlights.

- `pages/play.rs`: `PlayPage` owns `game: RwSignal<Game>` and
  `selected: RwSignal<Option<Square>>` and passes both to
  `<ChessBoard game selected/>`. A lowercase status line from
  `status_text(&Game)` (the user likes it lowercase), and "new game" / "undo"
  buttons that also clear `selected`; undo has `disabled=cant_undo`.
- Engine `Display`: `Color` (`white`), `Move` (UCI), `WinReason`,
  `DrawReason` (lowercase words, read as "{winner} wins by {reason}").
- `components/board.rs`:
  - `glyph(Piece) -> char` and `PieceIcon` (span with `piece white`/`piece
    black`) are everything about how a piece looks; SVG pieces later only
    change these.
  - `BoardSquare(sq, game, selected)`: named derived closures above the view
    (`is_selected`, `is_target`, `is_capture`, `is_last`, `is_check`, `piece`,
    `on_click`), then a `div` with `class="square"` and `class:` toggles
    (`dark`, `light`, `selected`, `target`, `capture`, `last_move`, `check`).
  - `ChessBoard(game, selected, perspective: RwSignal<Color>)`: a `move ||`
    closure renders `Square::reading_order(perspective.get())` as 64
    `BoardSquare`s, so a flip rebuilds the list.
  - `click_square`: plays `find_move(from, sq, None)` or the queen promotion,
    otherwise selects via `can_select`.
- `style/main.css`: 64px grid, design colours, dot via `.target::after`, ring
  via `.capture::after`, `color-mix` for last move and check.
- Engine helpers added for the UI: `Square::is_dark`, `Square::reading_order(Color)`
  (one pipeline, `file ^ mask` / `rank ^ mask` with mask 0 or 7), `Game::find_move`, `targets`,
  `can_select`, `last_move`. `Game` caches its legal moves in a `legal` field
  (refreshed on `play`/`undo`, emptied when the game ends) and
  `legal_moves()` returns `&MoveList`. `Game` methods are ordered:
  constructors, queries, actions, private helpers.

Covered so far: components and props, `view!` with Rust inside `{ }`,
attributes and `class:` toggles, lists via `collect_view()`, `Option`
rendering nothing, signals (`with`/`update`/`get`/`set`), why `move ||`
matters (a value is read once, a closure is re-run), derived signals, events,
lifting state up, CSS grid, `::after`, specificity, `color-mix`.

Style the user likes: logic in named closures above the `view!`, the view
itself a flat list of facts. Chess questions are answered by the engine, the
UI only maps answers to classes and clicks. Keep state as low as possible.

Gotchas:
- If a change seems to have no effect in the browser, hard reload
  (`Ctrl+Shift+R`) first: the browser can cache old CSS or WASM.
- Editor: the user's Neovim (AstroNvim) formats Rust through rust-analyzer
  with `leptosfmt --stdin --rustfmt`, so `view!` macros get indented on save.
  Suggest writing outer tags first, then filling in.
- Dots are hard to see on dark squares; consider a lighter dot there.
- The user found the pace close to overwhelming; the rest of milestone 2
  reuses the same concepts, so say so and keep steps small.

Coordinates: `BoardSquare` gets a plain `side: Color` prop; rank labels on
the left file and file labels on the bottom rank (`sq.file() ^ mask == 0`),
as `.coord` spans positioned absolutely. `PlayPage` owns
`perspective: RwSignal<Color>` and a "flip" button.

Step 6 parts: 6a status line (done), 6b new game / undo (done), 6c flip
(done),
6d move list (UCI via `Move` `Display`), 6e layout per the design.

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
