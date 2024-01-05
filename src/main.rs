use std::time::Instant;
use std::{thread, time,  env};
use std::str::FromStr;
use std::sync::mpsc;
use chess::{self, ChessMove, Square};

mod uci;
mod search;

fn main() {
    uci::uci_mode();
    
    //text_ui_mode();
    
}

fn text_ui_mode() {
    let mut board = chess::Board::default();

    let args: Vec<String> = env::args().collect();
    let mut max_depth = 99;
    let mut time_limit = 1;
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
        break;
    }
    
    if fen_string.len() > 0 {
        board = chess::Board::from_str(&fen_string).expect("Invalid Position");
    }

    println!("FEN: {board}");
    
    loop {
        let (stop_sender, stop_receiver) = mpsc::channel();
        let (info_sender, info_receiver) = mpsc::channel();
        let mut context = search::SearchContext::new(board, stop_receiver, info_sender);

        let now = Instant::now();

        let handle = thread::spawn(move || {
            return context.root_search(max_depth);
        });

        let search_duration = time::Duration::from_millis(1000*time_limit);
        let update_interval = time::Duration::from_millis(10);
        let mut last_checkpoint = now.elapsed();

        loop {
            thread::sleep(update_interval);

            let (score, best_move, depth) = info_receiver.try_recv().unwrap_or(
                (0, ChessMove::new(Square::A1, Square::A1, None), 0)
            );
            
            if depth > 0 {
                
                let depth_duration = now.elapsed() - last_checkpoint;
                last_checkpoint = now.elapsed();
                println!("d{depth} | {best_move} | {score} | {:.2?}", depth_duration);
            }
            if now.elapsed() > search_duration {break;}
        }

        for _ in 0..100 {
            let _ = stop_sender.send(true);
        }

        let result = handle.join().expect("Valid Search Result");


        let elapsed = now.elapsed();
        let score = result.0;
        let best_move = result.1;
        board = board.make_move_new(best_move);
        

        println!("Elapsed: {:.2?}", elapsed);
        println!("Result of search: {score}");
        println!("Best move: {best_move}");
        println!("FEN: {board}");


        let user_input = uci::collect_user_input();

        let user_move = chess::ChessMove::from_str(&user_input).expect("Invalid move:");

        board  = board.make_move_new(user_move);
        
    }
}

