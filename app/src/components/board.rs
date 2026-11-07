use engine::{
    game::Game,
    piece::{Color, Piece, PieceKind},
    square::Square,
};
use leptos::prelude::*;

fn glyph(piece: Piece) -> char {
    match piece.kind() {
        PieceKind::Pawn => '♟',
        PieceKind::Knight => '♞',
        PieceKind::Bishop => '♝',
        PieceKind::Rook => '♜',
        PieceKind::Queen => '♛',
        PieceKind::King => '♚',
    }
}

#[component]
fn PieceIcon(piece: Piece) -> impl IntoView {
    let class = match piece.color() {
        Color::White => "piece white",
        Color::Black => "piece black",
    };
    view! { <span class=class>{glyph(piece)}</span> }
}

fn click_square(game: RwSignal<Game>, selected: RwSignal<Option<Square>>, sq: Square) {
    if let Some(from) = selected.get()
        && let Some(mv) = game.with(|g| {
            g.find_move(from, sq, None)
                .or_else(|| g.find_move(from, sq, Some(PieceKind::Queen)))
        })
    {
        game.update(|g| {
            let _ = g.play(mv);
        });
        selected.set(None);
        return;
    }

    let own = game.with(|g| g.can_select(sq));
    selected.set(own.then_some(sq));
}

#[component]
fn BoardSquare(
    sq: Square,
    game: RwSignal<Game>,
    selected: RwSignal<Option<Square>>,
    side: Color,
) -> impl IntoView {
    let is_dark = sq.is_dark();
    let is_selected = move || selected.get() == Some(sq);
    let is_target = move || {
        selected
            .get()
            .is_some_and(|from| game.with(|g| g.targets(from).contains(sq)))
    };
    let is_capture = move || is_target() && game.with(|g| g.board().piece_at(sq).is_some());
    let is_last = move || {
        game.with(|g| {
            g.last_move()
                .is_some_and(|mv| mv.from() == sq || mv.to() == sq)
        })
    };
    let is_check = move || {
        game.with(|g| {
            g.board().in_check() && g.board().king_square(g.board().side_to_move()) == Some(sq)
        })
    };
    let piece = move || {
        game.with(|g| g.board().piece_at(sq))
            .map(|piece| view! { <PieceIcon piece /> })
    };

    let on_click = move |_| click_square(game, selected, sq);

    let mask = match side {
        Color::White => 0,
        Color::Black => 7,
    };

    let rank_label = (sq.file() ^ mask == 0).then(|| sq.rank() + 1);
    let file_label = (sq.rank() ^ mask == 0).then(|| (b'a' + sq.file()) as char);

    view! {
        <div
            class="square"
            class:dark=is_dark
            class:light=!is_dark
            class:selected=is_selected
            class:target=is_target
            class:capture=is_capture
            class:last_move=is_last
            class:check=is_check
            on:click=on_click
        >
            {piece}
            {rank_label.map(|r| view! { <span class="coord rank">{r}</span> })}
            {file_label.map(|f| view! { <span class="coord file">{f}</span> })}
        </div>
    }
}

#[component]
pub fn ChessBoard(
    game: RwSignal<Game>,
    selected: RwSignal<Option<Square>>,
    perspective: RwSignal<Color>,
) -> impl IntoView {
    let squares = move || {
        let side = perspective.get();
        Square::reading_order(side)
            .map(|sq| view! { <BoardSquare sq game selected side /> })
            .collect_view()
    };
    view! { <div class="board">{squares}</div> }
}
