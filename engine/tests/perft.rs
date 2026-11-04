//! Perft: counts every position reachable to a fixed depth and compares with
//! published numbers. One wrong move anywhere in move generation or
//! `make_move` changes the count, so this covers the whole move pipeline.
//!
//! Positions and numbers: <https://www.chessprogramming.org/Perft_Results>

use engine::{board::Board, movegen::perft};

#[allow(
    clippy::expect_used,
    reason = "test helper: a bad FEN here should fail the test loudly"
)]
fn check(fen: &str, expected: &[u64]) {
    let board = Board::from_fen(fen).expect("valid FEN");
    for (depth, &want) in (1..).zip(expected) {
        assert_eq!(perft(&board, depth), want, "{fen} at depth {depth}");
    }
}

#[test]
fn start_position() {
    check(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        &[20, 400, 8_902, 197_281, 4_865_609],
    );
}

#[test]
fn kiwipete() {
    check(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        &[48, 2_039, 97_862, 4_085_603],
    );
}

#[test]
fn endgame_en_passant() {
    check(
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        &[14, 191, 2_812, 43_238, 674_624],
    );
}

#[test]
fn promotions_and_checks() {
    check(
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        &[6, 264, 9_467, 422_333],
    );
}

#[test]
fn discovered_checks() {
    check(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        &[44, 1_486, 62_379, 2_103_487],
    );
}
