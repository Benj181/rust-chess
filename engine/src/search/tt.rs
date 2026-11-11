//! The transposition table: a cache of positions the search has already
//! scored, found by their Zobrist hash.

use crate::types::Move;

const SLOTS: usize = 131_072;

/// How an entry's score relates to the true score of its position.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    /// The score is exact.
    Exact,
    /// The true score is at least this (the search was cut off early).
    Lower,
    /// The true score is at most this (no move raised alpha).
    Upper,
}

#[derive(Clone, Copy)]
pub struct Entry {
    pub key: u64,
    pub depth: u32,
    pub score: i32,
    pub bound: Bound,
    pub best: Move,
}

pub struct Table {
    slots: Vec<Option<Entry>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            slots: vec![None; SLOTS],
        }
    }

    fn index(&self, key: u64) -> Option<usize> {
        let len = u64::try_from(self.slots.len()).ok()?;
        usize::try_from(key.checked_rem(len)?).ok()
    }

    pub fn probe(&self, key: u64) -> Option<Entry> {
        let entry = (*self.slots.get(self.index(key)?)?)?;
        (entry.key == key).then_some(entry)
    }

    pub fn store(&mut self, entry: Entry) {
        if let Some(index) = self.index(entry.key)
            && let Some(slot) = self.slots.get_mut(index)
        {
            *slot = Some(entry);
        }
    }
}
