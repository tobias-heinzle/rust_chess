use std::str::FromStr;
use std::io;
use std::env;
use chess::MoveGen;
use rust_chess;
use chess;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut board = chess::Board::default();//::from_str("r4r1k/1R1R2p1/7p/8/8/3Q1Ppq/P7/6K1 w - - 0 1").expect("Invalid Position");
    let max_depth = 8;


    let fen_string = args[1..].join(&" ");
    if fen_string.len() > 0 {
        board = chess::Board::from_str(&fen_string).expect("Invalid Position");
    }

    loop {
        let now = Instant::now();
        let result = rust_chess::root_search(&board, max_depth);
        let elapsed = now.elapsed();
    
        let score = result.0;
        let best_move = result.1;
    
        println!("Elapsed: {:.2?}", elapsed);
        println!("Depth: {max_depth}");
        println!("Result of search: {score}");
        println!("Best move: {best_move}");

        let mut result = chess::Board::default();

        board.make_move(best_move, &mut result);

        board = result;

        let mut user_input = String::new();
        match io::stdin().read_line(&mut user_input) {
            Ok(_) => {},
            Err(_) => {},
        }
        user_input.trim().to_string();
        user_input.remove(4);

        let user_move = chess::ChessMove::from_str(&user_input).expect("Invalid move:");

        let mut result = chess::Board::default();

        board.make_move(user_move, &mut result);

        board = result;
        
    }
    
}