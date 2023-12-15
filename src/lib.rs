
use std::cmp::max;
use std::time::Instant;
use chess::{Board, MoveGen, Piece, Color, ALL_PIECES, ChessMove, Square, BoardStatus, EMPTY, BitBoard};


type ScoreType = i32;
type SearchResult = (ScoreType, ChessMove);
type MoveOrdering = Vec<fn(&Board) -> Option<BitBoard>>;

const INFINITY: i32 = 1000000;
const PIECE_VALUES: [i32; 6] = [80, 300, 305, 450, 900, INFINITY];
const PIN_VALUE: i32 = 10;
const MOBILITY_VALUE: i32 = 1;
const IN_CHECK_PENALTY: i32 = 30;

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

    // Color dependent evaluation
    let all_white =  board.color_combined(Color::White);
    let all_black =  board.color_combined(Color::Black);

    for piece in ALL_PIECES {
        
        let white = board.pieces(piece) & all_white;
        let black = board.pieces(piece) & all_black;

        score += PIECE_VALUES[i] * (white.popcnt() as i32 - black.popcnt() as i32);

        i += 1;

    }

    score += PIN_VALUE * (board.pinned() & all_black).popcnt() as i32;
    score -= PIN_VALUE * (board.pinned() & all_white).popcnt() as i32;

    
    if board.side_to_move() == Color::Black {
        score = -score;
    }

    // evaluation independent of color
    if *board.checkers() == EMPTY{
        score += MOBILITY_VALUE * MoveGen::movegen_perft_test(board, 1) as i32;
        score -= MOBILITY_VALUE * MoveGen::movegen_perft_test(&board.null_move().expect("Valid Position"), 1) as i32;
    }
    else {
        score -= IN_CHECK_PENALTY;
    }

    return score
    
}

pub fn root_search(board: &Board, max_depth: u8) -> SearchResult{
    let mut iterable = MoveGen::new_legal(board);

    let mut best_move = ChessMove::new(Square::A1, Square::A1, None);

    let mut move_list: Vec<ChessMove> = vec![];
    for get_targets in mvv_ordering() {
            
        let targets = get_targets(board).unwrap_or(EMPTY);
        iterable.set_iterator_mask(targets);
    
        for chess_move in &mut iterable{
            move_list.push(chess_move);
        }
    }     
    
    let mut alpha = -INFINITY;

    for depth in 1..(max_depth + 1) {
        let now = Instant::now();

        alpha = -INFINITY;
        let beta = -alpha;

        if depth > 1{
            let result = board.make_move_new(best_move);
            
            alpha = -search(&result, depth -1, -beta, -alpha);
            if alpha >= 100000 {
                return (alpha, best_move)
            }
        }
        
        for chess_move in &mut move_list {

            if *chess_move == best_move { continue; }

            let result = board.make_move_new(*chess_move);
            
            let value = -search(&result, depth -1, -beta, -alpha);
            if value > alpha {
                best_move = *chess_move;
                alpha = value;

                if value >= 100000 {
                    return (alpha, best_move)
                }
            }
        }

        let elapsed = now.elapsed();
        println!("d{depth} | {alpha} | {best_move} | {:.2?}", elapsed)
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

            let result = board.make_move_new(chess_move);
           
            value = max(value, -search(&result, depth -1, -beta, -alpha));
            alpha = max(alpha, value);
            if alpha >= beta { break; }
            
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