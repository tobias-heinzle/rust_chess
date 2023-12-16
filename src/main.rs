use std::str::FromStr;
use std::io;
use std::env;
use rust_chess;
use chess;
use std::time::Instant;

mod nn;

fn main() {
    
    let mut board = chess::Board::default();

    let args: Vec<String> = env::args().collect();
    let mut max_depth = 5;
    let mut fen_string: String = "".to_string();

    if args.len() > 1 {
        if args[1] == "-d" {
            max_depth = args[2].parse::<u8>().unwrap_or(0);
            fen_string = args[3..].join(&" ");
        }
        else{
            fen_string = args[1..].join(&" ");
        }
    
        
    }
    
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