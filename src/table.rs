use crate::search::{PositionScore, SearchDepth};
// use chess::CacheTable;
use chess::ChessMove;
use std::sync::{Arc, RwLock};

// pub type TranspositionTable = CacheTable<TableEntryData>;

pub type TranspositionTable = SharedTable<TableEntryData>;

// TODO: Add the flag PV

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum ScoreBound {
    Exact,
    UpperBound,
    LowerBound,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct TableEntryData {
    pub depth: SearchDepth,
    pub score_bound: ScoreBound,
    pub score: PositionScore,
    pub best_move: ChessMove,
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
struct TableEntry<T: Copy + Clone + PartialEq + PartialOrd> {
    hash: u32,
    data: T,
}

pub struct SharedTable<T: Copy + Clone + PartialEq + PartialOrd> {
    table: Arc<Vec<RwLock<TableEntry<T>>>>,
    mask: usize,
}

impl<T: Copy + Clone + PartialEq + PartialOrd> SharedTable<T> {
    #[inline]
    pub fn new(size: usize, default: T) -> SharedTable<T> {
        let values: Vec<_> = (0..size)
            .map(|_| {
                RwLock::new(TableEntry {
                    hash: 0,
                    data: default,
                })
            })
            .collect();

        SharedTable {
            table: Arc::new(values),
            mask: size - 1,
        }
    }

    #[inline]
    pub fn get(&self, hash: u64) -> Option<T> {
        let idx = (hash as usize) & self.mask;
        let entry = self.table[idx].read().unwrap();
        if entry.hash == hash as u32 {
            Some(entry.data)
        } else {
            None
        }
    }

    #[inline]
    pub fn add(&self, hash: u64, entry: T) {
        let idx = (hash as usize) & self.mask;
        let mut table_entry = self.table[idx].write().unwrap();

        table_entry.hash = hash as u32;
        table_entry.data = entry;
    }

    #[inline(always)]
    pub fn replace_if<F: Fn(T) -> bool>(&self, hash: u64, entry: T, replace: F) {
        let idx = (hash as usize) & self.mask;
        let mut table_entry = self.table[idx].write().unwrap();
        if replace(table_entry.data) {
            table_entry.hash = hash as u32;
            table_entry.data = entry
        }
    }
}

impl<T: Copy + Clone + PartialEq + PartialOrd> Clone for SharedTable<T> {
    fn clone(&self) -> Self {
        SharedTable {
            table: self.table.clone(),
            mask: self.mask,
        }
    }
}
