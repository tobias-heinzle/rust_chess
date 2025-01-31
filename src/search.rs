use chess::{BitBoard, Board, BoardStatus, ChessMove, Color, MoveGen, Piece, Square, EMPTY};
use derive_new::new;
use std::cmp::max;
use std::ops::Index;
use std::rc::Rc;
use std::sync::mpsc;

use crate::config::{self, MAX_DEPTH, MAX_EXTENSION_PLIES};
use crate::eval::evaluate;
use crate::movelist::MoveList;
use crate::table::{ScoreBound, TableEntryData, TranspositionTable};

pub type PositionScore = i32;
pub type SearchDepth = u8;
pub type SearchOutcome = (PositionScore, ChessMove);
pub type SearchInfo = (PositionScore, ChessMove, SearchDepth);

// TODO: Add History Heuristic

// TODO: instead of alpha, beta etc. pass an object that encapsulates a search state

const MAX_HISTORY_VALUE: u16 = 65535 - (MAX_DEPTH as u16 * MAX_DEPTH as u16 + 1);

#[derive(Clone, Copy)]
pub struct KillerMoves {
    pub one: ChessMove,
    pub two: ChessMove,
}

impl KillerMoves {
    #[inline]
    fn store(&mut self, chess_move: ChessMove) {
        if chess_move != self.one {
            self.two = self.one;
            self.one = chess_move;
        }
    }
}

pub type Table64by64 = [[u16; 64]; 64];

struct HistoryTables {
    white: Table64by64,
    black: Table64by64,
}

impl Index<Color> for HistoryTables {
    type Output = Table64by64;
    fn index(&self, color: Color) -> &Self::Output {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }
}

impl HistoryTables {
    fn new() -> HistoryTables {
        HistoryTables {
            white: [[0; 64]; 64],
            black: [[0; 64]; 64],
        }
    }

    fn increment(&mut self, chess_move: &ChessMove, color: Color, depth: SearchDepth) {
        match color {
            Color::White => {
                self.white[chess_move.get_source().to_index()][chess_move.get_dest().to_index()] +=
                    1;
                // if self.white[chess_move.get_source().to_index()][chess_move.get_dest().to_index()]
                //     >= MAX_HISTORY_VALUE
                // {
                //     self.divide_history_white();
                // }
            }
            Color::Black => {
                self.black[chess_move.get_source().to_index()][chess_move.get_dest().to_index()] +=
                    1;
                // if self.black[chess_move.get_source().to_index()][chess_move.get_dest().to_index()]
                //     >= MAX_HISTORY_VALUE
                // {
                //     self.divide_history_black();
                // }
            }
        }
    }

    fn divide_history_white(&mut self) {
        for from_square in 0..64 {
            for to_square in 0..64 {
                self.white[from_square][to_square] >>= 1;
            }
        }
    }

    fn divide_history_black(&mut self) {
        for from_square in 0..64 {
            for to_square in 0..64 {
                self.black[from_square][to_square] >>= 1;
            }
        }
    }
}

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

    #[new(value = "[0; config::REP_TABLE_SIZE]")]
    pub repetition_table: [u8; config::REP_TABLE_SIZE],
    #[new(value = "vec![]")]
    pub past_position_hashes: Vec<u64>,
    #[new(value = "config::MVV_ORDERING")]
    pub capture_order: [Piece; 5],
    #[new(value = "1")]
    pub start_depth: u8,
    #[new(value = "false")]
    terminate_search: bool,
    #[new(value = "vec![]")]
    killers: Vec<KillerMoves>,
    #[new(value = "[[0; 64]; 64]")] //"HistoryTables::new()")]
    history_tables: Table64by64, // HistoryTables,
}

impl SearchContext {
    pub fn root_search(&mut self, max_depth: SearchDepth) -> SearchOutcome {
        // TODO: order moves here with scores attached! (Hash moves gets good score, increase of alpha gets good score etc.)
        let mut move_vec = get_legal_moves_vector(&self.board);
        let mut best_move = move_vec[0];
        let mut score = -config::INFINITY;

        use rand::Rng;
        let mut rng = rand::thread_rng();

        for i in 0..64 {
            for j in 0..64 {
                self.history_tables[i][j] = rng.gen::<u16>()
            }
        }

        let dummy_move = ChessMove::new(Square::A1, Square::A1, None);

        self.killers = vec![
            KillerMoves {
                one: dummy_move.clone(),
                two: dummy_move.clone()
            };
            (max_depth + MAX_EXTENSION_PLIES) as usize
        ];

        self.set_visited(self.board.get_hash());

        'iterative_deepening: for depth in self.start_depth..(max_depth + 1) {
            let mut current_best = best_move;

            // TODO: Add aspiration windows

            let mut alpha = -config::INFINITY;

            move_vec.sort_by_key(|m| if best_move.eq(m) { 0 } else { 1 });

            for chess_move in &mut move_vec {
                // TODO: only search first move with full window, later moves with zero window

                let value = -self.search(
                    &self.board.make_move_new(*chess_move),
                    depth - 1,
                    -config::INFINITY,
                    -alpha,
                    0,
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

            // TODO: report also the PV once implemented (Read from TTable)

            self.sender_channel
                .send((score, best_move, depth))
                .unwrap_or_default();
        }

        self.unset_visited(self.board.get_hash());

        (score, best_move)
    }

    pub fn search(
        &mut self,
        board: &Board,
        mut depth: SearchDepth,
        mut alpha: PositionScore,
        mut beta: PositionScore,
        mut plies_extended: SearchDepth,
        ply: usize,
    ) -> PositionScore {
        if self.terminate_search {
            return alpha;
        }

        if self.already_visited(board.get_hash()) {
            return config::DRAW;
        }

        if depth <= 0 || board.status() != BoardStatus::Ongoing {
            return self.quiescence_search(board, alpha, beta);
        }

        if extend_check(board, plies_extended) {
            depth += 1;
            plies_extended += 1;
        }

        // TODO: try if just ChessMove performs better than option type
        let mut hash_move: Option<ChessMove> = None;
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

            hash_move = Some(table_entry.best_move);
        }

        let movelist = MoveList::new(board, hash_move, self.killers[ply], self.history_tables);

        let mut score_bound = ScoreBound::UpperBound;
        let mut best_move = ChessMove::new(Square::A1, Square::A1, None);

        self.set_visited(board.get_hash());

        for chess_move in movelist {
            // TODO: only search first move with full window, later moves with zero window

            let mut value = -self.search(
                &board.make_move_new(chess_move),
                depth - 1,
                -beta,
                -alpha,
                plies_extended,
                ply + 1,
            );

            if value > config::MATE_THRESHOLD {
                value -= 1;
            }

            if value > alpha {
                best_move = chess_move;
                alpha = value;
                score_bound = ScoreBound::Exact;

                if alpha >= beta {
                    // TODO if movegen.stage == quiet

                    match board.piece_on(chess_move.get_dest()) {
                        Some(_) => {}
                        None => {
                            // if depth >= 2 {
                            // self.history_tables
                            //     .increment(&chess_move, board.side_to_move(), depth);
                            // }
                            // self.history_tables[chess_move.get_source().to_index()]
                            //     [chess_move.get_dest().to_index()] += depth as u16;
                            self.killers[ply].store(chess_move);
                        }
                    }
                    score_bound = ScoreBound::LowerBound;
                    break;
                }
            }
        }

        self.unset_visited(board.get_hash());

        if self.receiver_channel.try_recv().unwrap_or(false) {
            self.terminate_search = true;
        }

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
                old_entry.depth <= depth || table_entry.score_bound == ScoreBound::Exact
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
            BoardStatus::Checkmate => return -config::INFINITY,
            BoardStatus::Stalemate => return config::DRAW,
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
        for piece in config::QS_ORDERING {
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
        if self.repetition_table[position_hash as usize % config::REP_TABLE_SIZE] >= 1 {
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
        self.repetition_table[position_hash as usize % config::REP_TABLE_SIZE] += 1;
        self.past_position_hashes.push(position_hash);
    }

    #[inline]
    pub fn unset_visited(&mut self, position_hash: u64) {
        self.repetition_table[position_hash as usize % config::REP_TABLE_SIZE] -= 1;
        self.past_position_hashes.pop();
    }
}

#[inline]
pub fn extend_check(board: &chess::Board, plies_extended: SearchDepth) -> bool {
    (*board.checkers() != EMPTY) && (plies_extended < config::MAX_EXTENSION_PLIES)
}

#[inline]
fn get_legal_moves_vector(board: &Board) -> Vec<ChessMove> {
    let mut iterable = MoveGen::new_legal(board);

    let mut move_vec: Vec<ChessMove> = vec![];
    for piece in config::MVV_ORDERING {
        iterable.set_iterator_mask(get_targets(board, piece));
        for chess_move in &mut iterable {
            move_vec.push(chess_move);
        }
    }

    iterable.set_iterator_mask(!EMPTY);
    for chess_move in &mut iterable {
        move_vec.push(chess_move);
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
