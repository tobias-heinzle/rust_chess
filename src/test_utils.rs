use crate::search::SearchContext;
use crate::table::{ScoreBound, TableEntryData, TranspositionTable};
use crate::uci::HASH_TABLE_SIZE;
use chess::{Board, ChessMove, Square};
use std::sync::mpsc::channel;

pub fn setup_test_context(board: Board) -> SearchContext {
    let (_, rx) = channel();
    let (tx, _) = channel();
    let hash_table = TranspositionTable::new(
        HASH_TABLE_SIZE as usize,
        TableEntryData {
            best_move: ChessMove::new(Square::A1, Square::A1, None),
            score: 0,
            depth: 0,
            score_bound: ScoreBound::LowerBound,
        },
    );

    SearchContext::new(board, rx, tx, hash_table)
}
