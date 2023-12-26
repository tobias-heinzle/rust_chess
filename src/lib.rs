
pub mod uci;
pub mod search;

// type ScoreType = i32;
// type DepthType = u8;
// pub type SearchResult = (ScoreType, ChessMove);
// pub type SearchInfo = (ScoreType, ChessMove, DepthType);

// pub const INFINITY: i32 = 1000000;
// const PIECE_VALUES: [i32; 6] = [80, 300, 305, 450, 900, INFINITY];
// const PIN_VALUE: i32 = 10;
// const MOBILITY_VALUE: i32 = 1;
// const IN_CHECK_PENALTY: i32 = 30;
// const MVV_ORDERING: [Piece; 6] = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight, Piece::Pawn, Piece::King];
// const QS_ORDERING: [Piece; 5] = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight, Piece::Pawn];

// pub struct SearchContext {
//     pub board: Board,
//     pub receiver_channel: mpsc::Receiver<bool>,
//     pub sender_channel: mpsc::Sender<SearchInfo>,

// }

// impl SearchContext {

//     pub fn root_search(&self, max_depth: DepthType) -> SearchResult{
    

//         let mut best_move = ChessMove::new(Square::A1, Square::A1, None);
//         let mut move_vec = get_legal_moves_vector(& self.board);
//         let mut score = -INFINITY;
    
//         for depth in 1..(max_depth + 1) {
//             let mut current_best = best_move;
//             let mut alpha = -INFINITY;
    
//             if depth > 1{
//                 alpha = - self.search(& self.board.make_move_new(best_move), depth -1, -INFINITY, -alpha);
//             }
            
//             for chess_move in &mut move_vec {
//                 if *chess_move == best_move { continue; }


//                 let value = -self.search(& self.board.make_move_new(*chess_move), depth -1, -INFINITY, -alpha);
    
    
//                 if value > alpha {
//                     current_best = *chess_move;
//                     alpha = value;
//                 }
//                 if self.receiver_channel.try_recv().unwrap_or(false){return (score, best_move) }
//             }

//             score = alpha;
//             best_move = current_best;
//             self.sender_channel.send((score, best_move, depth)).unwrap_or_default();
//         }
        
//         return (score, best_move)
//     }



//     pub fn search(&self, board: &Board, depth: DepthType, mut alpha: ScoreType, beta: ScoreType) -> ScoreType{

//         if depth <= 0 || board.status() != BoardStatus::Ongoing{
//             return self.quiescence_search(board, alpha, beta)
//         }
    
//         let mut iterable = MoveGen::new_legal(board);
    
//         for piece in MVV_ORDERING {
//             if self.receiver_channel.try_recv().unwrap_or(false){return alpha; }

//             iterable.set_iterator_mask( get_targets(&board, piece ));
    
//             for chess_move in &mut iterable{
    
//                 let value = - self.search(&board.make_move_new(chess_move), depth - 1, -beta, -alpha);
    
                
//                 alpha = max(alpha, value);
                
                
//                 if alpha >= beta { break; }
                
//             }
//         }
    
//         return alpha
//     }


//     pub fn quiescence_search(&self, board: &Board, mut alpha: ScoreType, beta: ScoreType) -> i32{

//         match board.status() {
//             BoardStatus::Checkmate => return -INFINITY,
//             BoardStatus::Stalemate => return 0,
//             _ => {}
//         }
        

//         alpha = max(evaluate(board), alpha);
//         if alpha >= beta { return beta };


//         let mut iterable = MoveGen::new_legal(board);

//         for piece in QS_ORDERING {

//             iterable.set_iterator_mask( get_targets(board, piece) );


//             for chess_move in &mut iterable{

            
//                 let value = - self.quiescence_search(&board.make_move_new(chess_move), -beta, -alpha);


//                 alpha = max(alpha, value);


//                 if alpha >= beta {return alpha;}
                
//             }
//         }

//         return alpha;

//     }
// }


// #[inline]
// fn evaluate(board: &Board) -> ScoreType{
//     let mut score = 0;

//     let all_player =  player_pieces(board);
//     let all_opponent =  opponent_pieces(board);
    

//     for (i, piece) in ALL_PIECES.iter().enumerate() {
        
//         let player = board.pieces(*piece) & all_player;
//         let opponent = board.pieces(*piece) & all_opponent;

//         score += PIECE_VALUES[i] * (player.popcnt() as i32 - opponent.popcnt() as i32);
//     }

//     score += PIN_VALUE * (board.pinned() & all_opponent).popcnt() as i32;
//     score -= PIN_VALUE * (board.pinned() & all_player).popcnt() as i32;

//     if *board.checkers() == EMPTY{
//         score += MOBILITY_VALUE * MoveGen::movegen_perft_test(board, 1) as i32;
//         score -= MOBILITY_VALUE * MoveGen::movegen_perft_test(&board.null_move().expect("Valid Position"), 1) as i32;
//     }
//     else {
//         score -= IN_CHECK_PENALTY;
//     }

//     return score;

// }


// #[inline]
// fn get_legal_moves_vector(board: &Board) -> Vec<ChessMove>{
//     let mut iterable = MoveGen::new_legal(board);

//     let mut move_vec: Vec<ChessMove> = vec![];
//     for piece in MVV_ORDERING {
            
//         iterable.set_iterator_mask( get_targets(board, piece) );
//         for chess_move in &mut iterable{ move_vec.push(chess_move); }
//     }     
//     return move_vec
// }

// #[inline]
// fn get_targets(board: &Board, pieces: Piece) -> BitBoard{
//     if (pieces == Piece::Pawn) && board.en_passant().is_some(){
//         return opponent_pieces(board) & BitBoard :: from_square(board.en_passant().unwrap());
//     }
//     else if pieces == Piece::King {
//         return !EMPTY;
//     }
//     return opponent_pieces(board) & board.pieces(pieces);
// }


// #[inline]
// fn opponent_pieces(board: &Board) -> BitBoard {
//     return *board.color_combined(!board.side_to_move()); 
// }

// #[inline]
// fn player_pieces(board: &Board) -> BitBoard {
//     return *board.color_combined(board.side_to_move()); 
// }
