use crate::state::PlayState;
use engine::{Color, Piece, PieceKind, Square};
use leptos::{ev, prelude::*};

#[derive(Clone, Copy)]
struct Drag {
    from: RwSignal<Option<Square>>,
    at: RwSignal<(i32, i32)>,
}

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
pub fn PieceIcon(piece: Piece) -> impl IntoView {
    let class = match piece.color() {
        Color::White => "piece white",
        Color::Black => "piece black",
    };
    view! { <span class=class>{glyph(piece)}</span> }
}

fn click_square(state: PlayState, sq: Square) {
    if !state.is_live() || state.ai_to_move() {
        return;
    }
    let PlayState { game, selected, .. } = state;
    if let Some(from) = selected.get() {
        if let Some(mv) = game.with(|g| g.find_move(from, sq, None)) {
            game.update(|g| {
                let _ = g.play(mv);
            });
            selected.set(None);
            return;
        }
        if game.with(|g| g.find_move(from, sq, Some(PieceKind::Queen)).is_some()) {
            state.promoting.set(Some((from, sq)));
            return;
        }
    }

    let own = game.with(|g| g.can_select(sq));
    selected.set(own.then_some(sq));
}

#[component]
fn BoardSquare(sq: Square, side: Color, drag: Drag) -> impl IntoView {
    let state = expect_context::<PlayState>();
    let PlayState {
        game,
        selected,
        shown: board,
        shown_move: last,
        ..
    } = state;
    let is_dark = sq.is_dark();
    let is_selected = move || selected.get() == Some(sq);
    let is_target = move || {
        selected
            .get()
            .is_some_and(|from| game.with(|g| g.targets(from).contains(sq)))
    };
    let is_capture = move || is_target() && board.with(|b| b.piece_at(sq).is_some());
    let is_last = move || {
        last.get()
            .is_some_and(|mv| mv.from() == sq || mv.to() == sq)
    };
    let is_check = move || {
        board.with(|b| b.in_check() && b.king_square(b.side_to_move()) == Some(sq))
    };
    let piece = move || {
        board.with(|b| b.piece_at(sq).map(|piece| view! { <PieceIcon piece /> }))
    };

    let on_click = move |_| click_square(state, sq);
    let start_drag = move |e: ev::PointerEvent| {
        if e.button() != 0 || !state.is_live() || !game.with(|g| g.can_select(sq)) {
            return;
        }
        selected.set(Some(sq));
        drag.at.set((e.client_x(), e.client_y()));
        drag.from.set(Some(sq));
    };
    let is_drag_from = move || drag.from.get() == Some(sq);

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
            class:drag_from=is_drag_from
            data-square=sq.to_string()
            on:click=on_click
            on:pointerdown=start_drag
        >
            {piece}
            {rank_label.map(|r| view! { <span class="coord rank">{r}</span> })}
            {file_label.map(|f| view! { <span class="coord file">{f}</span> })}
        </div>
    }
}

const PROMOTION_CHOICES: [PieceKind; 4] = [
    PieceKind::Queen,
    PieceKind::Knight,
    PieceKind::Rook,
    PieceKind::Bishop,
];

#[component]
fn PromotionPicker() -> impl IntoView {
    let state = expect_context::<PlayState>();
    move || {
        let (_, to) = state.promoting.get()?;
        let color = state.game.with(|g| g.board().side_to_move());
        let mask = match state.perspective.get() {
            Color::White => 0,
            Color::Black => 7,
        };
        let column = (to.file() ^ mask) + 1;
        let at_top = to.rank() ^ mask == 7;
        let rows = if at_top { "1 / span 4" } else { "5 / span 4" };

        let choices = PROMOTION_CHOICES
            .map(|kind| {
                let piece = Piece::new(color, kind);
                view! {
                    <button class="promotion-choice" on:click=move |_| state.promote(kind)>
                        <PieceIcon piece />
                    </button>
                }
            })
            .collect_view();

        Some(view! {
            <div class="promotion-layer" on:click=move |_| state.cancel_promotion()>
                <div
                    class="promotion"
                    class:from-bottom=!at_top
                    class:white=color == Color::White
                    style:grid-column=column.to_string()
                    style:grid-row=rows
                >
                    {choices}
                </div>
            </div>
        })
    }
}

fn square_at(x: i32, y: i32) -> Option<Square> {
    let element = document().element_from_point(x as f32, y as f32)?;
    let square = element.closest("[data-square]").ok()??;
    Square::from_algebraic(&square.get_attribute("data-square")?)
}

#[component]
fn DragGhost(drag: Drag) -> impl IntoView {
    let board = expect_context::<PlayState>().shown;
    move || {
        let from = drag.from.get()?;
        let piece = board.with(|b| b.piece_at(from))?;
        Some(view! {
            <div
                class="drag-ghost"
                style:left=move || format!("{}px", drag.at.get().0)
                style:top=move || format!("{}px", drag.at.get().1)
            >
                <PieceIcon piece />
            </div>
        })
    }
}

#[component]
pub fn ChessBoard() -> impl IntoView {
    let state = expect_context::<PlayState>();
    let perspective = state.perspective;
    let drag = Drag {
        from: RwSignal::new(None),
        at: RwSignal::new((0, 0)),
    };

    let follow = window_event_listener(ev::pointermove, move |e| {
        if drag.from.get().is_some() {
            drag.at.set((e.client_x(), e.client_y()));
        }
    });
    let drop = window_event_listener(ev::pointerup, move |e| {
        let Some(from) = drag.from.get() else {
            return;
        };
        drag.from.set(None);
        if let Some(to) = square_at(e.client_x(), e.client_y())
            && to != from
        {
            click_square(state, to);
        }
    });
    on_cleanup(move || {
        follow.remove();
        drop.remove();
    });

    let squares = move || {
        let side = perspective.get();
        Square::reading_order(side)
            .map(|sq| view! { <BoardSquare sq side drag /> })
            .collect_view()
    };
    view! {
        <div class="board" on:click=|ev| ev.stop_propagation()>
            {squares}
            <PromotionPicker />
            <DragGhost drag />
        </div>
    }
}
