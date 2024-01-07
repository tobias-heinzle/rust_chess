use std::thread::JoinHandle;
use std::{thread, time, io};
use std::sync::mpsc::{self, Receiver, Sender};
use std::str::FromStr;
use chess::{self, ChessMove, Square};

use crate::search::{SearchInfo, SearchResult, SearchContext};

const STOP_SIGNAL: bool = true;

// TODO: Change this code to use the Game Struct from chess crate   

struct SearchThread {
    handle: JoinHandle<SearchResult>,
    termination_sender: Sender<bool>
}

pub fn uci_mode(){
    let mut board = chess::Board::default();
    let mut position_hashes: Vec<u64> = vec![];
    let mut is_searching = false;

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
        else if command == "ucinewgame" { board = chess::Board::default(); position_hashes = vec![]; }
        else if command == "position"   { (board, position_hashes) = change_position(arguments); }
        else if command == "stop"       { terminate_search(search_threads); search_threads = vec![]; is_searching = false; }
        else if command == "quit"       { terminate_search(search_threads); break;}
        else if command == "go"         { 
            if is_searching {
                continue;
            } 
            else { 
                // TODO: Accept movetime parameter!
                search_threads = start_search_threads(1, board, info_sender.clone(), position_hashes.clone());
                is_searching = true; 
            }
        }
        else                            { log(format!("bad input: {input_line}"));}

    }

    let _ = stop_print_sender.send(STOP_SIGNAL);
    let _ = output_thread_handle.join();

}

fn start_output_thread() -> (Sender<String>, Sender<SearchInfo>, Sender<bool>, JoinHandle<()>) {
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


fn start_search_threads(n_workers: u8, board: chess::Board,  info_sender: Sender<SearchInfo>, position_hashes: Vec<u64>) -> Vec<SearchThread> {
    let mut search_threads:  Vec<SearchThread> = vec![];
    let (dummy_sender, _) = mpsc::channel();

    for i in 0 .. n_workers {
        let (stop_sender, stop_receiver) = mpsc::channel();
        let mut context = SearchContext::new(
            board, 
            stop_receiver, 
            if i == 0 { info_sender.clone() } else { dummy_sender.clone() });
        
        for hash in position_hashes.iter() {
            context.set_visited(*hash);
        }

        let thread_handle = thread::spawn(move || context.root_search(200));
        
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
        let thread_result = thread.handle.join();
        if thread_result.is_ok() {
            results.push(thread_result.unwrap());
        }
    }

    let (score, best_move) = results[0];

    println!("info score cp {score}");
    println!("bestmove {best_move}");
}

fn log(text: String) {
    println!("{text}");
}

pub fn change_position(arguments: &[&str]) -> (chess::Board, Vec<u64>){
    let mut new_board = chess::Board::default();
    let mut hash_vec: Vec<u64> = vec![];

    let moves_index = arguments.iter().position(|&r| r == "moves").unwrap_or(arguments.len());

    if arguments[0] == "fen" {
        let fen_string = arguments[1 .. moves_index].join(" ");
        new_board = chess::Board::from_str(&fen_string).unwrap_or(new_board);
    }

    hash_vec.push(new_board.get_hash());

    for move_str in &arguments[moves_index .. ]{
        let parsed_move_result = chess::ChessMove::from_str(move_str);

        if parsed_move_result.is_ok() {
            // TODO: Figure out why this sometimes returns an error value and deal with it
            let move_obj = parsed_move_result.unwrap();
            new_board = new_board.make_move_new(move_obj);
            hash_vec.push(new_board.get_hash());
        }

    }

    return (new_board, hash_vec); 
}


pub fn collect_user_input() -> String{
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => {},
        Err(_) => {},
    }
    return user_input.trim().to_string();
}

fn printing_loop(info_receiver: Receiver<SearchInfo>, print_reveiver: Receiver<String>, terminate_print_receiver: Receiver<bool>){
    let update_interval = time::Duration::from_millis(10);

    loop {
        thread::sleep(update_interval);

        let message = print_reveiver.try_recv().unwrap_or("".to_string());
        if message.len() > 0 { println!("{message}"); }

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