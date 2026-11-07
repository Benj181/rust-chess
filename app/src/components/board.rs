use engine::{
    game::Game,
    moves::MoveKind,
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

fn click_square(game: RwSignal<Game>, selected: RwSignal<Option<Square>>, sq: Square) {
    if let Some(from) = selected.get() 
        && let Some(mv) = game.with(|g| {
            g.legal_moves()
                .iter()
                .find(|mv| {
                    mv.from() == from
                        && mv.to() == sq
                        && matches!(mv.kind().promotion(), None | Some(PieceKind::Queen))
                })
                .copied()
        }) {
            game.update(|g| {
                let _ = g.play(mv);
            });
            selected.set(None);
            return;
        }
    
        if game.with(|g| {
            g.board()
                .piece_at(sq)
                .is_some_and(|p| p.color() == g.board().side_to_move())
        }) {
            selected.set(Some(sq));
        } else {
            selected.set(None);
        }
        
}

#[component]
pub fn ChessBoard(game: RwSignal<Game>) -> impl IntoView {
    let selected = RwSignal::new(None::<Square>);

    let squares = (0..8).rev().flat_map(|rank| {
        (0..8).map(move |file| {
            let dark = (file ^ rank) & 1 == 0;
            let sq = Square::from_file_rank(file, rank);

            view! {
                <div
                    class=if dark { "square dark" } else { "square light" }
                    class:selected=move || selected.get() == sq
                    on:click=move |_| {
                        if let Some(sq) = sq {
                            click_square(game, selected, sq)
                        }
                    }
                >
                    {move || {
                        game.with(|g| sq.and_then(|sq| g.board().piece_at(sq)))
                            .map(|p| {
                                view! {
                                    <span class=if p.color() == Color::White {
                                        "piece white"
                                    } else {
                                        "piece black"
                                    }>{glyph(p)}</span>
                                }
                            })
                    }}
                </div>
            }
        })
    });

    view! { <div class="board">{squares.collect_view()}</div> }
}
