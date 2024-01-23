
use std::sync::mpsc::channel;
use chess::{Board, ChessMove, Square};
use crate::search::SearchContext;
use crate::table::{TranspositionTable, TableEntryData, ScoreBound};
use crate::uci::HASH_TABLE_SIZE;

pub fn setup_test_context(board: Board) -> SearchContext {
    let (_, rx) = channel();
    let (tx, _) = channel();
    let hash_table = TranspositionTable::new(
        HASH_TABLE_SIZE, 
        TableEntryData{
            best_move : ChessMove::new(
                Square::A1, 
                Square::A1, 
                None), 
            score : 0, 
            depth : 0, 
            score_bound : ScoreBound::LowerBound}
    );
    
    return SearchContext::new(board, rx, tx, hash_table);
}