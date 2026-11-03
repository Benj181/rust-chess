#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const NONE: Self = Self(0);
    pub const WHITE_KING: Self = Self(0b0001);
    pub const WHITE_QUEEN: Self = Self(0b0010);
    pub const BLACK_KING: Self = Self(0b0100);
    pub const BLACK_QUEEN: Self = Self(0b1000);
    pub const ALL: Self = Self(0b1111);

    #[must_use]
    pub fn from_fen(s: &str) -> Option<Self> {
        let mut rights = Self::NONE;
        if s == "-" {
            return Some(rights);
        }
        for c in s.chars() {
            let right = match c {
                'K' => Self::WHITE_KING,
                'Q' => Self::WHITE_QUEEN,
                'k' => Self::BLACK_KING,
                'q' => Self::BLACK_QUEEN,
                _ => return None,
            };
            rights.insert(right);
        }
        Some(rights)
    }

    #[must_use]
    pub const fn contains(self, right: Self) -> bool {
        self.0 & right.0 == right.0
    }

    pub const fn insert(&mut self, right: Self) {
        self.0 |= right.0;
    }

    pub const fn remove(&mut self, right: Self) {
        self.0 &= !right.0;
    }
}
