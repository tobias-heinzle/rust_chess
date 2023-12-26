use std::env::join_paths;
use std::thread::JoinHandle;
use std::time::Instant;
use std::{thread, time, io, env};
use std::sync::mpsc::{self, Receiver, Sender};
use std::str::FromStr;
use rust_chess::{self, SearchInfo, SearchResult, SearchContext};
use chess::{self, ChessMove, Square};

mod nn;

const STOP_SIGNAL: bool = true;

struct SearchThread {
    handle: JoinHandle<SearchResult>,
    termination_sender: Sender<bool>
}

fn main() {
    uci_mode()
    
    //text_ui_mode();
    
}

fn uci_mode(){
    let mut board = chess::Board::default();

    let (print_sender, 
        info_sender, 
        stop_print_sender, 
        output_thread_handle) = start_output_thread();
    let mut search_threads: Vec<SearchThread> = vec![];
    
    let respond = |message: &str| {let _ = print_sender.send(message.to_string());};

    loop {
        let input_line = collect_user_input();
        let input: Vec<&str> = input_line.split(" ").collect();
        let command = input[0];
        let arguments = &input[1..];

        if      command == "uci"        { respond("uciok"); }
        else if command == "isready"    { respond("readyok"); }
        else if command == "ucinewgame" { board = chess::Board::default(); }
        else if command == "position"   { board = change_position(arguments); }
        else if command == "go"         { search_threads = start_search_threads(2, board, info_sender.clone()); }
        else if command == "stop"       { terminate_search(search_threads); search_threads = vec![] }
        else if command == "quit"       { terminate_search(search_threads); break;}
        else                            { log(format!("bad input: {input_line}"));}

    }

    let _ = stop_print_sender.send(STOP_SIGNAL);
    let _ = output_thread_handle.join();

}

fn start_output_thread() -> (Sender<String>, Sender<rust_chess::SearchInfo>, Sender<bool>, JoinHandle<()>) {
    let (print_sender, print_receiver) = mpsc::channel();
    let (info_sender, info_receiver) = mpsc::channel();
    let (stop_sender, stop_receiver) = mpsc::channel();


    let output_thread_handle = thread::spawn(
        move || printing_loop(
            info_receiver, 
            print_receiver, 
            stop_receiver));

    return (print_sender, info_sender, stop_sender, output_thread_handle);
}


fn start_search_threads(n_workers: u8, board: chess::Board,  info_sender: Sender<SearchInfo>) -> Vec<SearchThread> {
    let mut search_threads:  Vec<SearchThread> = vec![];

    for _ in 0 .. n_workers {
        let (stop_sender, stop_receiver) = mpsc::channel();
        let context = SearchContext{board: board, receiver_channel: stop_receiver, sender_channel: info_sender.clone()};
        let thread_handle = thread::spawn(move || context.root_search(99));
        
        let search_thread = SearchThread{handle: thread_handle, termination_sender: stop_sender};

        search_threads.push(search_thread);

    }

    return search_threads;
}

fn send_termination_signal(sender: &Sender<bool>, n_signals: i32) {
    for _ in 0 .. n_signals { 
        let _ = sender.send(STOP_SIGNAL); 
    }
}

fn terminate_search(threads: Vec<SearchThread>) {

    if threads.len() == 0 { return; }

    for thread in &threads {
        send_termination_signal(&thread.termination_sender, 100);
    }

    let mut results = vec![];

    for thread in threads {
        results.push(thread.handle.join().unwrap());
    }

    let (score, best_move) = results[0];

    println!("info score cp {score}");
    println!("bm {best_move}");
}

fn log(text: String) {
    println!("{text}");
}

fn change_position(arguments: &[&str]) -> chess::Board{

    if arguments[0] == "fen" {
        let mut fen_string = "";

        

        for word in arguments {
            if *word != "moves" {fen_string = "{fen_string} {word}";}
            else { break; }
        }

        println!("{fen_string}");

    }

    let new_board = chess::Board::from_str("df");

    return new_board.unwrap_or(chess::Board::default()); 
}


fn collect_user_input() -> String{
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => {},
        Err(_) => {},
    }
    return user_input.trim().to_string();
}

fn printing_loop(info_receiver: Receiver<rust_chess::SearchInfo>, print_reveiver: Receiver<String>, terminate_print_receiver: Receiver<bool>){
    let update_interval = time::Duration::from_millis(10);

    loop {
        thread::sleep(update_interval);

        let message = print_reveiver.try_recv().unwrap_or("".to_string());

        if message.len() > 0 {
            println!("{message}");
        }

        let (score, best_move, depth) = info_receiver.try_recv().unwrap_or(
            (0, ChessMove::new(Square::A1, Square::A1, None), 0)
        );
        
        if depth > 0 {
            
            println!("info depth {depth} score cp {score} pv {best_move}");
        }

        let termination_signal = terminate_print_receiver.try_recv().unwrap_or(false);

        if termination_signal { return }

    }

}


// fn text_ui_mode() {
//     let mut board = chess::Board::default();

//     let args: Vec<String> = env::args().collect();
//     let mut max_depth = 99;
//     let mut time_limit = 1;
//     let mut fen_string: String = "".to_string();

//     let mut skip_arg = false;
//     for (i, arg) in args[1..].iter().enumerate(){
//         if skip_arg {
//             skip_arg = false;
//             continue;
//         }
//         if args.len() > i{
//             if arg == "-d" {
//                 max_depth = args[i + 2].parse::<u8>().unwrap_or(5);
//                 skip_arg = true;
//                 continue;
//             }
//             else if arg == "-t" {
//                 time_limit = args[i + 2].parse::<u64>().unwrap_or(5);
//                 skip_arg = true;
//                 continue;
//             }
//         }

//         fen_string = args[i + 1 ..].join(&" ");
//         break;
//     }
    
//     if fen_string.len() > 0 {
//         board = chess::Board::from_str(&fen_string).expect("Invalid Position");
//     }

//     println!("FEN: {board}");
    
//     loop {
//         let (stop_sender, stop_receiver) = mpsc::channel();
//         let (info_sender, info_receiver) = mpsc::channel();
//         let context = rust_chess::SearchContext{board: board, receiver_channel: stop_receiver, sender_channel: info_sender};

//         let now = Instant::now();

//         let handle = thread::spawn(move || {
//             return context.root_search(max_depth);
//         });

//         let search_duration = time::Duration::from_millis(1000*time_limit);
//         let update_interval = time::Duration::from_millis(10);
//         let mut last_checkpoint = now.elapsed();

//         loop {
//             thread::sleep(update_interval);

//             let (score, best_move, depth) = info_receiver.try_recv().unwrap_or(
//                 (0, ChessMove::new(Square::A1, Square::A1, None), 0)
//             );
            
//             if depth > 0 {
                
//                 let depth_duration = now.elapsed() - last_checkpoint;
//                 last_checkpoint = now.elapsed();
//                 println!("d{depth} | {best_move} | {score} | {:.2?}", depth_duration);
//             }
//             if now.elapsed() > search_duration {break;}
//         }

//         for _ in 0..100 {
//             let _ = stop_sender.send(STOP_SIGNAL);
//         }

//         let result = handle.join().expect("Valid Search Result");


//         let elapsed = now.elapsed();
//         let score = result.0;
//         let best_move = result.1;
//         board = board.make_move_new(best_move);
        

//         println!("Elapsed: {:.2?}", elapsed);
//         println!("Result of search: {score}");
//         println!("Best move: {best_move}");
//         println!("FEN: {board}");


//         let user_input = collect_user_input();

//         let user_move = chess::ChessMove::from_str(&user_input).expect("Invalid move:");

//         board  = board.make_move_new(user_move);
        
//     }
// }

