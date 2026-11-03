#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Square(u8);

impl Square {
    #[must_use]
    pub const fn new(index: u8) -> Option<Self> {
        if index >= 64 {
            return None;
        }

        Some(Self(index))
    }

    #[must_use]
    pub fn from_file_rank(file: u8, rank: u8) -> Option<Self> {
        if file >= 8 {
            return None;
        }
        Self::new(rank.checked_mul(8)?.checked_add(file)?)
    }

    #[must_use]
    pub fn from_algebraic(s: &str) -> Option<Self> {
        let mut chars = s.chars();
        let file = "abcdefgh".find(chars.next()?)?;
        let rank = "12345678".find(chars.next()?)?;
        if chars.next().is_some() {
            return None;
        }
        Self::from_file_rank(u8::try_from(file).ok()?, u8::try_from(rank).ok()?)
    }

    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn file(self) -> u8 {
        self.0 % 8
    }

    #[must_use]
    pub const fn rank(self) -> u8 {
        self.0 / 8
    }
}
