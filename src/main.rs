use std::str::FromStr;
use rust_chess;
use chess;
use std::time::Instant;

fn main() {
    
    let board = chess::Board::from_str("r4r2/1R1R2pk/7p/8/8/5Ppq/P7/6K1 w - - 0 2").expect("Invalid Position");
    let max_depth = 6;


    let now = Instant::now();
    let result = rust_chess::root_search(&board, max_depth);
    let elapsed = now.elapsed();

    let score = result.0;
    let best_move = result.1;

    println!("Elapsed: {:.2?}", elapsed);
    println!("Depth: {max_depth}");
    println!("Result of search: {score}");
    println!("Best move: {best_move}")
    
}