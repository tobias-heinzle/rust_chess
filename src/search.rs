
use std::cmp::max;
use std::sync::mpsc;
use derive_new::new;
use chess::{Board, MoveGen, Piece, ChessMove, Square, BoardStatus, EMPTY, BitBoard};

type ScoreType = i32;
type DepthType = u8;
pub type SearchResult = (ScoreType, ChessMove);
pub type SearchInfo = (ScoreType, ChessMove, DepthType);

// Evaluation constants
pub const INFINITY: i32 = 1000000;
pub const DRAW: i32 = 0;

const PAWN_VALUE: i32 = 80;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 305;
const ROOK_VALUE: i32 = 450;
const QUEEN_VALUE: i32 = 900;
// const PIECE_VALUES: [i32; 6] = [PAWN_VALUE, KNIGHT_VALUE, BISHOP_VALUE, ROOK_VALUE, QUEEN_VALUE, INFINITY];

const PIN_VALUE: i32 = 10;
const MOBILITY_VALUE: i32 = 1;
const IN_CHECK_PENALTY: i32 = 30;

// Move ordering
const MVV_ORDERING: [Piece; 6] = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight, Piece::Pawn, Piece::King];
const QS_ORDERING: [Piece; 5] = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight, Piece::Pawn];

// Repetition detection
const REP_TABLE_SIZE: usize = 1 << 16;

#[derive(new)]
pub struct SearchContext {
    pub board: Board,
    pub receiver_channel: mpsc::Receiver<bool>,
    pub sender_channel: mpsc::Sender<SearchInfo>,

    #[new(value = "[0; REP_TABLE_SIZE]")]
    pub repetition_table: [u8; REP_TABLE_SIZE],

} 

impl SearchContext {

    pub fn root_search(&mut self, max_depth: DepthType) -> SearchResult{

        let mut best_move = ChessMove::new(Square::A1, Square::A1, None);
        let mut move_vec = get_legal_moves_vector(& self.board);
        let mut score = -INFINITY;

        self.set_visited(self.board.get_hash());
    
        for depth in 1..(max_depth + 1) {
            let mut current_best = best_move;
            let mut alpha = -INFINITY;
    
            if depth > 1{
                alpha = - self.search(& self.board.make_move_new(best_move), depth -1, -INFINITY, -alpha);
            }
            
            for chess_move in &mut move_vec {
                if *chess_move == best_move { continue; }


                let value = -self.search(& self.board.make_move_new(*chess_move), depth -1, -INFINITY, -alpha);
    
    
                if value > alpha {
                    current_best = *chess_move;
                    alpha = value;
                }
                if self.receiver_channel.try_recv().unwrap_or(false){return (score, best_move) }
            }

            score = alpha;
            best_move = current_best;
            self.sender_channel.send((score, best_move, depth)).unwrap_or_default();
        }

        self.unset_visited(self.board.get_hash());
        return (score, best_move)
    }



    pub fn search(&mut self, board: &Board, depth: DepthType, mut alpha: ScoreType, beta: ScoreType) -> ScoreType{

        if depth <= 0 || board.status() != BoardStatus::Ongoing{
            return self.quiescence_search(board, alpha, beta)
        }

        if self.already_visited(board.get_hash()){
            return DRAW;
        }
        
        self.set_visited(board.get_hash());
       
        let mut iterable = MoveGen::new_legal(board);
    
        'outer: for piece in MVV_ORDERING {
            if self.receiver_channel.try_recv().unwrap_or(false){return alpha; }

            iterable.set_iterator_mask( get_targets(&board, piece ));
    
            for chess_move in &mut iterable{
    
                let value = - self.search(&board.make_move_new(chess_move), depth - 1, -beta, -alpha);
    
                
                alpha = max(alpha, value);
                
                
                if alpha >= beta { break 'outer; }
                
            }
        }

        self.unset_visited(board.get_hash());

        return alpha
    }


    pub fn quiescence_search(&mut self, board: &Board, mut alpha: ScoreType, beta: ScoreType) -> i32{

        match board.status() {
            BoardStatus::Checkmate => return -INFINITY,
            BoardStatus::Stalemate => return DRAW,
            _ => {}
        }
        
        if self.already_visited(board.get_hash()){
            return DRAW;
        }
        
        alpha = max(evaluate(board), alpha);
        if alpha >= beta { return beta };
        
        self.set_visited(board.get_hash());

        let mut iterable = MoveGen::new_legal(board);

        'outer: for piece in QS_ORDERING {

            iterable.set_iterator_mask( get_targets(board, piece) );


            for chess_move in &mut iterable{

            
                let value = - self.quiescence_search(&board.make_move_new(chess_move), -beta, -alpha);


                alpha = max(alpha, value);


                if alpha >= beta {break 'outer}
                
            }
        }

        self.unset_visited(board.get_hash());
        return alpha;

    }

    pub fn already_visited(&mut self, position_hash: u64) -> bool{
        if self.repetition_table[position_hash as usize % REP_TABLE_SIZE] == 1{
            return true;
        }
        return false
    }
    
    pub fn set_visited(&mut self, position_hash: u64) {
        self.repetition_table[position_hash as usize % REP_TABLE_SIZE] = 1;
    }

    pub fn unset_visited(&mut self, position_hash: u64) {
        self.repetition_table[position_hash as usize % REP_TABLE_SIZE] = 0;
    }


}


#[inline]
fn evaluate(board: &Board) -> ScoreType{
    let mut score = 0;

    let all_player =  player_pieces(board);
    let all_opponent =  opponent_pieces(board);
    let blockers = all_player | all_opponent;

    // Pawn evaluation
    let mut player = board.pieces(Piece::Pawn) & all_player;
    let mut opponent = board.pieces(Piece::Pawn) & all_opponent;

    score += PAWN_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    // Knight evalutation
    player = board.pieces(Piece::Knight) & all_player;
    opponent = board.pieces(Piece::Knight) & all_opponent;

    score += KNIGHT_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);
    
    for square in player {
        score += MOBILITY_VALUE * chess::get_knight_moves(square).popcnt() as i32;
    }
    for square in opponent {
        score -= MOBILITY_VALUE * chess::get_knight_moves(square).popcnt() as i32;
    }

    // Bishop evalutation
    player = board.pieces(Piece::Bishop) & all_player;
    opponent = board.pieces(Piece::Bishop) & all_opponent;

    score += BISHOP_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);
    
    for square in player {
        score += MOBILITY_VALUE * chess::get_bishop_moves(square, blockers).popcnt() as i32;
    }
    for square in opponent {
        score -= MOBILITY_VALUE * chess::get_bishop_moves(square, blockers).popcnt() as i32;
    }

    // Rook evalutation
    player = board.pieces(Piece::Rook) & all_player;
    opponent = board.pieces(Piece::Rook) & all_opponent;

    score += ROOK_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);
    
    for square in player {
        score += MOBILITY_VALUE * chess::get_rook_moves(square, blockers).popcnt() as i32;
    }
    for square in opponent {
        score -= MOBILITY_VALUE * chess::get_rook_moves(square, blockers).popcnt() as i32;
    }

    // Queen evaluation
    player = board.pieces(Piece::Queen) & all_player;
    opponent = board.pieces(Piece::Queen) & all_opponent;

    score += QUEEN_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    score += PIN_VALUE * (board.pinned() & all_opponent).popcnt() as i32;
    score -= PIN_VALUE * (board.pinned() & all_player).popcnt() as i32;

    if *board.checkers() != EMPTY{
        score -= IN_CHECK_PENALTY;
    }

    return score;

}


#[inline]
fn get_legal_moves_vector(board: &Board) -> Vec<ChessMove>{
    let mut iterable = MoveGen::new_legal(board);

    let mut move_vec: Vec<ChessMove> = vec![];
    for piece in MVV_ORDERING {
            
        iterable.set_iterator_mask( get_targets(board, piece) );
        for chess_move in &mut iterable{ move_vec.push(chess_move); }
    }     
    return move_vec
}

#[inline]
fn get_targets(board: &Board, pieces: Piece) -> BitBoard{
    if (pieces == Piece::Pawn) && board.en_passant().is_some(){
        return opponent_pieces(board) & BitBoard :: from_square(board.en_passant().unwrap());
    }
    else if pieces == Piece::King {
        return !EMPTY;
    }
    return opponent_pieces(board) & board.pieces(pieces);
}


#[inline]
fn opponent_pieces(board: &Board) -> BitBoard {
    return *board.color_combined(!board.side_to_move()); 
}

#[inline]
fn player_pieces(board: &Board) -> BitBoard {
    return *board.color_combined(board.side_to_move()); 
}
