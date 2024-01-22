// use chess::ChessMove;
// use std::sync::atomic::{AtomicPtr, Ordering};
// use std::sync::Arc;

// use crate::search::{PositionScore, SearchDepth};

// #[derive(Clone, Copy, PartialEq, PartialOrd)]
// pub enum ScoreBound {
//     Exact,
//     UpperBound,
//     LowerBound,
// }

// #[derive(Clone, Copy, PartialEq, PartialOrd)]
// pub struct TableEntry {
//     pub hash : u64,
//     pub depth: SearchDepth,
//     pub score_bound: ScoreBound,
//     pub score: PositionScore,
//     pub best_move: ChessMove,
// }



// const ARRAY_SIZE: usize = 1 << 18;
// const ARRAY_MASK: usize = ARRAY_SIZE - 1;

// pub struct SharedArray<T> {
//     array : Box<[AtomicPtr<T>]>,
// }

// impl<T: Clone> SharedArray<T> {
//     // pub fn new() -> Self {
//     //     let mut array = Vec::with_capacity(ARRAY_SIZE);
//     //     for _ in 0..ARRAY_SIZE {
//     //         array.push(AtomicPtr::new(ptr::null_mut()));
//     //     }
//     //     SharedArray { array: array.into_boxed_slice() }
//     // }

//     pub fn new(default_value: T) -> Self {
//         let mut array = Vec::with_capacity(ARRAY_SIZE);

//         for _ in 0..ARRAY_SIZE {
//             array.push(AtomicPtr::new(Box::into_raw(Box::new(default_value.clone()))));
//         }

//         SharedArray { array: array.into_boxed_slice() }
//     }

//     #[inline]
//     pub fn get(&self, index: usize) -> Option<&T> {
//         let ptr = self.array[index & ARRAY_MASK].load(Ordering::Acquire);

//         Some(unsafe { &*ptr })
//     }

//     #[inline]
//     pub fn set(&self, index: usize, value: T) {

//         self.array[index & ARRAY_MASK].store(Box::into_raw(Box::new(value)), Ordering::Release);
    
//     }
// }

// impl<T> Drop for SharedArray<T> {
//     fn drop(&mut self) {
//         for ptr in self.array.iter() {
//             let _ = unsafe { Box::from_raw(ptr.load(Ordering::Relaxed)) };
//         }
//     }
// }



// pub struct TranspositionTable {
//     shared_table: Arc<SharedArray<TableEntry>>
// }

// impl TranspositionTable {
//     pub fn new(shared_array: Arc<SharedArray<TableEntry>>) -> Self {
//         TranspositionTable {
//             shared_table : shared_array,
//         }
//     }

//     #[inline]
//     pub fn get(&self, hash: u64) -> Option<TableEntry> {
//         let entry = self.shared_table.get(hash as usize).unwrap();
//         if entry.hash == hash {
//             return Some(*entry)
//         }
//         else {
//             return None
//         }
//     }

//     #[inline(always)]
//     pub fn replace_if<F: Fn(TableEntry) -> bool>(&self, hash: u64, entry: TableEntry, replace: F) {
//         let e = self.shared_table.get(hash as usize).unwrap();
//         if replace(*e) {
//             self.shared_table.set(hash as usize, entry);
//         }
//     }

//     #[inline]
//     pub fn add(&self, hash: u64, entry: TableEntry) {
//         self.shared_table.set(hash as usize, entry);
//     }
// }





use std::sync::{Arc, RwLock};

#[derive(Copy, Clone, PartialEq, PartialOrd)]
struct TableEntry<T: Copy + Clone + PartialEq + PartialOrd> {
    hash: u64,
    entry: T,
}

pub struct TranspositionTable<T: Copy + Clone + PartialEq + PartialOrd> {
    table: Arc<Vec<RwLock<TableEntry<T>>>>,
    mask: usize,
}

impl<T: Copy + Clone + PartialEq + PartialOrd> TranspositionTable<T> {
    #[inline]
    pub fn new(size: usize, default: T) -> TranspositionTable<T> {
        let values: Vec<_> = (0..size)
            .map(|_| RwLock::new(TableEntry { hash: 0, entry: default }))
            .collect();

        TranspositionTable {
            table: Arc::new(values),
            mask: size - 1,
        }
    }

    #[inline]
    pub fn get(&self, hash: u64) -> Option<T> {
        let idx = (hash as usize) & self.mask;
        let entry = self.table[idx].read().unwrap();
        if entry.hash == hash {
            Some(entry.entry)
        } else {
            None
        }
    }

    #[inline]
    pub fn add(&self, hash: u64, entry: T) {
        let idx = (hash as usize) & self.mask;
        let mut table_entry = self.table[idx].write().unwrap();

        table_entry.hash = hash;
        table_entry.entry = entry;
    }


    #[inline(always)]
    pub fn replace_if<F: Fn(T) -> bool>(&mut self, hash: u64, entry: T, replace: F) {
        let idx = (hash as usize) & self.mask;
        let mut table_entry = self.table[idx].write().unwrap();
        if replace(table_entry.entry) {
            table_entry.hash = hash;
            table_entry.entry = entry
        }
    }
}