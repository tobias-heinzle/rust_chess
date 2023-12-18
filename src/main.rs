use std::sync::mpsc;
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
    let mut time_limit = 5;
    let mut fen_string: String = "".to_string();

    let mut skip_arg = false;
    for (i, arg) in args[1..].iter().enumerate(){
        if skip_arg {
            skip_arg = false;
            continue;
        }
        if args.len() > i{
            if arg == "-d" {
                max_depth = args[i + 2].parse::<u8>().unwrap_or(5);
                skip_arg = true;
                continue;
            }
            else if arg == "-t" {
                time_limit = args[i + 2].parse::<u64>().unwrap_or(5);
                skip_arg = true;
                continue;
            }
        }

        fen_string = args[i + 1 ..].join(&" ");
    }
    
    if fen_string.len() > 0 {
        board = chess::Board::from_str(&fen_string).expect("Invalid Position");
    }
    
    loop {
        let (_, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let engine = rust_chess::ChessEngine{board: board, receiver_channel: rx, sender_channel: tx};

        let now = Instant::now();
        let result = engine.root_search(max_depth);
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


// fn process(input: String){
//     let command: Vec<&str> = input.split(" ").collect();
//     match command[0]{
//         "uci" => respond("uciok"),
//         "isready" => respond("readyok"),
//         "ucinewgame" => newgame(),
//         "position" => change_position(command[1..]),
//         "go" => start_search(command[1..]),
//         "stop" => stop_search(),
//         "quit" => {stop_search(); quit()},

//         _ => {log("bad cmd: {command}")}

//     }
// }