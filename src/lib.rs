
use std::cmp::max;

use chess::{Board, MoveGen, Piece, Color, ALL_PIECES, ChessMove, Square, BoardStatus, EMPTY, BitBoard};


type ScoreType = i32;
type SearchResult = (ScoreType, ChessMove);
type MoveOrdering = Vec<fn(&Board) -> Option<BitBoard>>;

const PIECE_VALUES: [i32; 6] = [80, 300, 305, 450, 900, -100000];
const QS_DELTA: i32 = 200;

fn no_ordering() -> MoveOrdering {
    return vec![|_board| Some(!EMPTY)];
}

fn mvv_ordering() -> MoveOrdering {
   return vec![
        |board| get_attacked(board, Piece::Queen),
        |board| get_attacked(board, Piece::Rook),
        |board| get_attacked(board, Piece::Bishop),
        |board| get_attacked(board, Piece::Knight),
        |board| get_attacked(board, Piece::Pawn),
        |board| get_en_passant(board),
        |_board| Some(!EMPTY),
        ];
}    
fn quiescence_ordering() -> MoveOrdering { 
    return vec![
        |board| get_attacked(board, Piece::Queen),
        |board| get_attacked(board, Piece::Rook),
        |board| get_attacked(board, Piece::Bishop),
        |board| get_attacked(board, Piece::Knight),
        ];
}
    

fn evaluate(board: &Board) -> ScoreType{
    let mut score = 0;
    let mut i: usize = 0;


    for piece in ALL_PIECES {
        
        let white = board.pieces(piece) & board.color_combined(Color::White);
        let black = board.pieces(piece) & board.color_combined(Color::Black);

        score += PIECE_VALUES[i] * (white.popcnt() as i32 - black.popcnt() as i32);

        i += 1;

    }
    if board.side_to_move() == Color::White {
        return score
    }

    return -score
    
}

pub fn root_search(board: &Board, max_depth: u8) -> SearchResult{
    let mut iterable = MoveGen::new_legal(board);

    let mut alpha = -100000;
    let beta = -alpha;
    let mut best_move = ChessMove::new(Square::A1, Square::A1, None);
    
    for get_targets in mvv_ordering() {
        
        let targets = get_targets(board).unwrap_or(EMPTY);

        iterable.set_iterator_mask(targets);
    
        for chess_move in &mut iterable{

            let mut result = Board::default();

            board.make_move(chess_move, &mut result);
            
            let value = -search(&result, max_depth -1, -beta, -alpha);

            if value > alpha {
                best_move = chess_move;
                alpha = value;
            }
            
        }
    }

    return (alpha, best_move)
}

fn search(board: &Board, depth: u8, mut alpha: ScoreType, beta: ScoreType) -> ScoreType{

    if depth == 0 || board.status() != BoardStatus::Ongoing{
        return quiescence_search(board, depth, alpha, beta)
    }
    
    let mut iterable = MoveGen::new_legal(board);
   
    let mut value = -100000;

    for get_targets in mvv_ordering() {
        
        let targets = get_targets(board).unwrap_or(EMPTY);

        iterable.set_iterator_mask(targets);

        for chess_move in &mut iterable{

            let mut result = Board::default();

            board.make_move(chess_move, &mut result);
            
            value = max(value, -search(&result, depth -1, -beta, -alpha));
            
            alpha = max(alpha, value);

            if alpha >= beta {
                break;
            }
            
        }
    }

    return value
}


pub fn quiescence_search(board: &Board, depth: u8, mut alpha: ScoreType, beta: ScoreType) -> i32{

    let board_status =  board.status();

    if board_status == BoardStatus::Checkmate{
        return -100000;
    }
    else if board_status == BoardStatus::Stalemate{ 
        return 0;
    }
    
    let baseline =  evaluate(board);

    return baseline;

}


fn get_attacked(board: &Board, pieces: Piece) -> Option<BitBoard>{
    return Some(board.color_combined(!board.side_to_move()) & board.pieces(pieces));
}

fn get_en_passant(board: &Board) -> Option<BitBoard>{
    let en_passant = board.en_passant();
    if en_passant.is_some(){
        return Some(board.color_combined(!board.side_to_move()) & BitBoard::from_square(en_passant.unwrap()));
    }
    else{
        return Some(EMPTY);
    }

}