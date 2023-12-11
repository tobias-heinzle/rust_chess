
use std::cmp::max;
use std::time::Instant;

use chess::{Board, MoveGen, Piece, Color, ALL_PIECES, ChessMove, Square, BoardStatus, EMPTY, BitBoard};


type ScoreType = i32;
type SearchResult = (ScoreType, ChessMove);
type MoveOrdering = Vec<fn(&Board) -> Option<BitBoard>>;

const PIECE_VALUES: [i32; 6] = [80, 300, 305, 450, 900, -100000];

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
    


fn main() {
    
    let board = Board::default(); // from_str("4r1k1/5bpp/2p5/3pr3/8/1B3pPq/PPR2P2/2R2QK1 b - - 0 1").expect("Invalid Position");
    let max_depth = 8;


    let now = Instant::now();
    let result = root_search(&board, max_depth);
    let elapsed = now.elapsed();

    let score = result.0;
    let best_move = result.1;

    println!("Elapsed: {:.2?}", elapsed);
    println!("Depth: {max_depth}");
    println!("Result of search: {score}");
    println!("Best move: {best_move}")
    
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
        return - quiescence_search(board, depth, alpha, beta)
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


fn quiescence_search(board: &Board, depth: u8, mut alpha: ScoreType, beta: ScoreType) -> i32{

    let board_status =  board.status();

    if board_status == BoardStatus::Checkmate{
        return -100000;
    }
    else if board_status == BoardStatus::Stalemate{ 
        return 0;
    }
    
    let baseline =  evaluate(board);

    return baseline;

    // if baseline >= beta {
    //     return beta;
    // }

    // if alpha < baseline {
    //     alpha = baseline;
    // }

    // let mut iterable = MoveGen::new_legal(board);

    // for get_targets in quiescence_ordering(){

    //     let targets = get_targets(board).unwrap_or(EMPTY);

    //     iterable.set_iterator_mask(targets);

    //     for chess_move in &mut iterable{

    //         let mut result = Board::default();

    //         board.make_move(chess_move, &mut result);
            
    //         alpha = max(alpha, -search(&result, depth -1, -beta, -alpha));

    //         if alpha >= beta {
    //             break;
    //         }

    //     }
    // }

    // return alpha
    
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