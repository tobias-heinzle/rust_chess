use chess::{BitBoard, Board, BoardStatus, ChessMove, MoveGen, Piece, Square, EMPTY};
use derive_new::new;
use std::cmp::max;
use std::sync::mpsc;

use crate::eval::evaluate;
use crate::table::{ScoreBound, TableEntryData, TranspositionTable};

pub type PositionScore = i32;
pub type SearchDepth = u8;
pub type SearchOutcome = (PositionScore, ChessMove);
pub type SearchInfo = (PositionScore, ChessMove, SearchDepth);

// Evaluation constants
pub const INFINITY: i32 = 1000000;
pub const DRAW: i32 = 0;
pub const MATE_MARGIN: i32 = 100;
pub const MATE_THRESHOLD: i32 = INFINITY - MATE_MARGIN;

// Move ordering; Most Valuable Victim first, King is a dummy value for quiet moves!
const MVV_ORDERING: [Piece; 6] = [
    Piece::Queen,
    Piece::Rook,
    Piece::Bishop,
    Piece::Knight,
    Piece::Pawn,
    Piece::King,
];
const QS_ORDERING: [Piece; 5] = [
    Piece::Queen,
    Piece::Rook,
    Piece::Bishop,
    Piece::Knight,
    Piece::Pawn,
];

// Repetition detection
const REP_TABLE_SIZE: usize = 1 << 16;

// Search Extension
const MAX_EXTENSION_PLIES: SearchDepth = 3;

// TODO: instead of alpha, beta etc. pass an object that encapsulates a search state

#[derive(new)]
pub struct SearchContext {
    // This struct contains all the information a thread needs search a position
    // In order to correctly identify a draw by threefold repetition, the SearchContext
    // needs to know the hashes of all previous positions. Also, after initializing, the
    // repetition_table needs to be incremented at the entries corresponding to these positions
    pub board: Board,
    pub receiver_channel: mpsc::Receiver<bool>,
    pub sender_channel: mpsc::Sender<SearchInfo>,
    pub hash_table: TranspositionTable,

    #[new(value = "[0; REP_TABLE_SIZE]")]
    pub repetition_table: [u8; REP_TABLE_SIZE],
    #[new(value = "vec![]")]
    pub past_position_hashes: Vec<u64>,
    #[new(value = "MVV_ORDERING")]
    pub move_ordering: [Piece; 6],
    #[new(value = "1")]
    pub start_depth: u8,
    #[new(value = "false")]
    terminate_search: bool,
}

impl SearchContext {
    pub fn root_search(&mut self, max_depth: SearchDepth) -> SearchOutcome {
        let mut move_vec = get_legal_moves_vector(&self.board);
        let mut best_move = move_vec[0];
        let mut score = -INFINITY;

        self.set_visited(self.board.get_hash());

        'iterative_deepening: for depth in self.start_depth..(max_depth + 1) {
            let mut current_best = best_move;
            let mut alpha = -INFINITY;

            move_vec.sort_by_key(|m| if best_move.eq(m) { 0 } else { 1 });

            for chess_move in &mut move_vec {
                let value = -self.search(
                    &self.board.make_move_new(*chess_move),
                    depth - 1,
                    -INFINITY,
                    -alpha,
                    0,
                );

                if value > alpha {
                    current_best = *chess_move;
                    alpha = value;
                }

                if self.terminate_search {
                    break 'iterative_deepening;
                }
            }

            score = alpha;
            best_move = current_best;
            self.sender_channel
                .send((score, best_move, depth))
                .unwrap_or_default();
        }

        self.unset_visited(self.board.get_hash());

        (score, best_move)
    }

    // TODO: think about LowerBound in TTable, is it needed? when does it occur?

    pub fn search(
        &mut self,
        board: &Board,
        mut depth: SearchDepth,
        mut alpha: PositionScore,
        mut beta: PositionScore,
        mut plies_extended: SearchDepth,
    ) -> PositionScore {
        if self.terminate_search {
            return alpha;
        }

        if self.already_visited(board.get_hash()) {
            return DRAW;
        }

        if depth <= 0 || board.status() != BoardStatus::Ongoing {
            return self.quiescence_search(board, alpha, beta);
        }

        if extend_check(board, plies_extended) {
            depth += 1;
            plies_extended += 1;
        }

        let mut best_move = ChessMove::new(Square::A1, Square::A1, None);
        let table_probe = self.hash_table.get(board.get_hash());

        if let Some(table_entry) = table_probe {
            if table_entry.depth >= depth {
                match table_entry.score_bound {
                    ScoreBound::Exact => {
                        if table_entry.score > alpha {
                            alpha = table_entry.score;
                            if alpha >= beta {
                                return beta;
                            }
                        }
                        return alpha;
                    }
                    ScoreBound::LowerBound => {
                        if table_entry.score > alpha {
                            alpha = table_entry.score
                        }
                    }
                    ScoreBound::UpperBound => {
                        if table_entry.score < beta {
                            beta = table_entry.score;
                        }
                    }
                }
            }

            if alpha >= beta {
                return beta;
            }

            if board.legal(table_entry.best_move) {
                self.set_visited(board.get_hash());

                let mut value = -self.search(
                    &board.make_move_new(table_entry.best_move),
                    depth - 1,
                    -beta,
                    -alpha,
                    plies_extended,
                );
                self.unset_visited(board.get_hash());

                if value > MATE_THRESHOLD {
                    value -= 1;
                }

                if value > alpha {
                    alpha = value;
                    if alpha >= beta {
                        return beta;
                    }
                }

                best_move = table_entry.best_move
            }
        }

        let mut score_bound = ScoreBound::UpperBound;
        let mut iterable = MoveGen::new_legal(board);

        self.set_visited(board.get_hash());

        'mvv_loop: for piece in self.move_ordering {
            //self.move_ordering {
            if self.receiver_channel.try_recv().unwrap_or(false) {
                self.terminate_search = true;
            }

            iterable.set_iterator_mask(get_targets(&board, piece));

            for chess_move in &mut iterable {
                if chess_move == best_move {
                    continue;
                }

                let mut value = -self.search(
                    &board.make_move_new(chess_move),
                    depth - 1,
                    -beta,
                    -alpha,
                    plies_extended,
                );

                if value > MATE_THRESHOLD {
                    value -= 1;
                }

                if value > alpha {
                    best_move = chess_move;
                    alpha = value;
                    score_bound = ScoreBound::Exact;

                    if alpha >= beta {
                        score_bound = ScoreBound::LowerBound;
                        break 'mvv_loop;
                    }
                }
            }
        }

        self.unset_visited(board.get_hash());

        if self.terminate_search {
            return alpha;
        }

        let table_entry = TableEntryData {
            best_move: best_move,
            score: alpha,
            depth: depth,
            score_bound: score_bound,
        };

        self.hash_table
            .replace_if(board.get_hash(), table_entry, |old_entry| {
                if old_entry.depth <= depth {
                    true
                } else {
                    false
                }
            });

        match score_bound {
            ScoreBound::LowerBound => beta,
            _ => alpha,
        }
    }

    pub fn quiescence_search(
        &mut self,
        board: &Board,
        mut alpha: PositionScore,
        mut beta: PositionScore,
    ) -> i32 {
        match board.status() {
            BoardStatus::Checkmate => return -INFINITY,
            BoardStatus::Stalemate => return DRAW,
            BoardStatus::Ongoing => {}
        }

        alpha = max(evaluate(board), alpha);

        if alpha >= beta {
            return beta;
        };

        let table_probe = self.hash_table.get(board.get_hash());
        if let Some(table_entry) = table_probe {
            match table_entry.score_bound {
                ScoreBound::Exact => {
                    alpha = table_entry.score;
                    if alpha >= beta {
                        return beta;
                    }
                    return alpha;
                }
                ScoreBound::LowerBound => {
                    if table_entry.score > alpha {
                        alpha = table_entry.score;
                    }
                }
                ScoreBound::UpperBound => {
                    if table_entry.score < beta {
                        beta = table_entry.score
                    }
                }
            }
            if alpha >= beta {
                return beta;
            }
        }

        let mut iterable = MoveGen::new_legal(board);
        for piece in QS_ORDERING {
            iterable.set_iterator_mask(get_targets(board, piece));

            for chess_move in &mut iterable {
                alpha = max(
                    alpha,
                    -self.quiescence_search(&board.make_move_new(chess_move), -beta, -alpha),
                );

                if alpha >= beta {
                    return beta;
                }
            }
        }

        alpha
    }

    #[inline]
    pub fn already_visited(&mut self, position_hash: u64) -> bool {
        if self.repetition_table[position_hash as usize % REP_TABLE_SIZE] >= 1 {
            for past_hash in self.past_position_hashes.iter() {
                if position_hash == *past_hash {
                    return true;
                }
            }
        }

        false
    }

    #[inline]
    pub fn set_visited(&mut self, position_hash: u64) {
        self.repetition_table[position_hash as usize % REP_TABLE_SIZE] += 1;
        self.past_position_hashes.push(position_hash);
    }

    #[inline]
    pub fn unset_visited(&mut self, position_hash: u64) {
        self.repetition_table[position_hash as usize % REP_TABLE_SIZE] -= 1;
        self.past_position_hashes.pop();
    }
}

#[inline]
pub fn extend_check(board: &chess::Board, plies_extended: SearchDepth) -> bool {
    if *board.checkers() == EMPTY {
        false
    } else if plies_extended < MAX_EXTENSION_PLIES {
        true
    } else {
        false
    }
}

#[inline]
fn get_legal_moves_vector(board: &Board) -> Vec<ChessMove> {
    let mut iterable = MoveGen::new_legal(board);

    let mut move_vec: Vec<ChessMove> = vec![];
    for piece in MVV_ORDERING {
        iterable.set_iterator_mask(get_targets(board, piece));
        for chess_move in &mut iterable {
            move_vec.push(chess_move);
        }
    }

    move_vec
}

#[inline]
fn get_targets(board: &Board, piece_type: Piece) -> BitBoard {
    match piece_type {
        Piece::Pawn => match board.en_passant() {
            Some(en_passant_square) => {
                opponent_pieces_of_type(piece_type, board)
                    | BitBoard::from_square(en_passant_square)
            }
            None => opponent_pieces_of_type(piece_type, board),
        },
        Piece::King => !EMPTY,
        _ => opponent_pieces_of_type(piece_type, board),
    }
}

#[inline]
pub fn opponent_pieces_of_type(piece_type: Piece, board: &Board) -> BitBoard {
    opponent_pieces(board) & board.pieces(piece_type)
}

#[inline]
pub fn opponent_pieces(board: &Board) -> BitBoard {
    *board.color_combined(!board.side_to_move())
}

#[inline]
pub fn player_pieces(board: &Board) -> BitBoard {
    *board.color_combined(board.side_to_move())
}
