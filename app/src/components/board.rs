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
pub fn ChessBoard() -> impl IntoView {
    let game = Game::new();
    let board = game.board();

    let squares = (0..8).rev().flat_map(|rank| {
        (0..8).map(move |file| {
            let dark = (file ^ rank) & 1 == 0;
            let piece = Square::from_file_rank(file, rank).and_then(|sq| board.piece_at(sq));
            view! {
                <div class=if dark {
                    "square dark"
                } else {
                    "square light"
                }>
                    {piece
                        .map(|p| {
                            view! {
                                <span class=if p.color() == Color::White {
                                    "piece white"
                                } else {
                                    "piece black"
                                }>{glyph(p)}</span>
                            }
                        })}
                </div>
            }
        })
    });

    view! { <div class="board">{squares.collect_view()}</div> }
}
