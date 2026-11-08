use engine::{
    board::Board,
    game::Game,
    moves::Move,
    piece::{Color, PieceKind},
    square::Square,
};
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct PlayState {
    pub game: RwSignal<Game>,
    pub selected: RwSignal<Option<Square>>,
    pub perspective: RwSignal<Color>,
    pub viewing: RwSignal<Option<usize>>,
    pub promoting: RwSignal<Option<(Square, Square)>>,
    pub shown_ply: Memo<usize>,
    pub shown: Memo<Board>,
    pub shown_move: Memo<Option<Move>>,
}

impl PlayState {
    pub fn provide() -> Self {
        let game = RwSignal::new(Game::new());
        let viewing = RwSignal::new(None);
        let shown_ply = Memo::new(move |_| {
            viewing
                .get()
                .unwrap_or_else(|| game.with(|g| g.moves().len()))
        });
        let shown = Memo::new(move |_| {
            let ply = shown_ply.get();
            game.with(|g| g.board_at(ply).unwrap_or(g.board()).clone())
        });
        let shown_move = Memo::new(move |_| {
            let ply = shown_ply.get();
            ply.checked_sub(1)
                .and_then(|i| game.with(|g| g.moves().get(i).copied()))
        });

        let state = Self {
            game,
            selected: RwSignal::new(None),
            perspective: RwSignal::new(Color::White),
            viewing,
            promoting: RwSignal::new(None),
            shown_ply,
            shown,
            shown_move,
        };
        provide_context(state);
        state
    }

    pub fn is_live(self) -> bool {
        self.viewing.with(Option::is_none)
    }

    pub fn view_ply(self, ply: usize) {
        let latest = self.game.with(|g| g.moves().len());
        self.cancel_promotion();
        self.viewing.set((ply < latest).then_some(ply));
    }

    pub fn promote(self, kind: PieceKind) {
        if let Some((from, to)) = self.promoting.get()
            && let Some(mv) = self.game.with(|g| g.find_move(from, to, Some(kind)))
        {
            self.game.update(|g| {
                let _ = g.play(mv);
            });
        }
        self.cancel_promotion();
    }

    pub fn cancel_promotion(self) {
        self.promoting.set(None);
        self.selected.set(None);
    }

    pub fn step_back(self) {
        self.view_ply(self.shown_ply.get().saturating_sub(1));
    }

    pub fn step_forward(self) {
        self.view_ply(self.shown_ply.get() + 1);
    }

    pub fn flip(self) {
        self.perspective.update(|p| *p = p.opposite());
    }

    pub fn go_live(self) {
        self.cancel_promotion();
        self.viewing.set(None);
    }

    pub fn load(self, game: Game) {
        self.game.set(game);
        self.go_live();
    }
}
