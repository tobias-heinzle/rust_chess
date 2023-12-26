use std::thread::JoinHandle;
use std::time::Instant;
use std::{thread, time, io, env};
use std::sync::mpsc::{self, Receiver, Sender};
use std::str::FromStr;
use rust_chess::{self, SearchInfo};
use chess::{self, ChessMove, Square};

mod nn;

const STOP_SIGNAL: bool = true;


fn main() {
    uci_mode()
    
    //text_ui_mode();
    
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


fn start_search_threads(n_workers: u8, board: chess::Board,  info_sender: Sender<SearchInfo>) ->Vec<Sender<bool>> {
    let mut stop_sender_vec:  Vec<Sender<bool>> = vec![];
    for _ in 0 .. n_workers {
        let (stop_sender, stop_receiver) = mpsc::channel();
        let context = rust_chess::SearchContext{board: board, receiver_channel: stop_receiver, sender_channel: info_sender.clone()};
        thread::spawn(move || context.root_search(99));
        stop_sender_vec.push(stop_sender)

    }

    return stop_sender_vec;
}

fn send_termination_signal(sender_vec: &Vec<Sender<bool>>, n_signals: i32) {
    for sender in sender_vec {
        for _ in 0 .. n_signals { 
            let _ = sender.send(STOP_SIGNAL); 
        }
    }
}

fn log(text: &str) {
    println!("{text}");
}

fn uci_mode(){
    let mut board = chess::Board::default();

    let (print_sender, 
        info_sender, 
        stop_print_sender, 
        output_thread_handle) = start_output_thread();
    let mut stop_search_vec: Vec<Sender<bool>> = vec![];
    
    let respond = |message: &str| {let _ = print_sender.send(message.to_string());};

    

    // let newgame = || board = chess::Board::default();

    // TODO: implement change_position
    let change_position = |arguments: &[&str]| return ();



    loop {
        let input_line = collect_user_input();
        let input: Vec<&str> = input_line.split(" ").collect();
        let command = input[0];
        let arguments = &input[1..];

        if      command == "uci"        { respond("uciok"); }
        else if command == "isready"    { respond("readyok"); }
        else if command == "ucinewgame" { board = chess::Board::default(); }
        else if command == "position"   { change_position(arguments); }
        else if command == "go"         { stop_search_vec = start_search_threads(2, board, info_sender.clone()); }
        else if command == "stop"       { send_termination_signal(&stop_search_vec, 100); }
        else if command == "quit"       { send_termination_signal(&stop_search_vec, 100); break;}
        else                            { log("bad input: {input}");}

        // let result: Option<Vec<Sender<bool>>> = match command[0]{
        //     "uci" => respond("uciok"),
        //     "isready" => respond("readyok"),
        //     //"ucinewgame" => newgame(),
        //     "position" => change_position(),// change_position(command[1..]),
        //     "go" => start_search_threads(2, board, info_sender.clone()),//start_search(command[1..]),
        //     "stop" => {for sender in stop_search_vec {send_termination_signal(sender, 100);} continue;},
        //     "quit" => {for sender in stop_search_vec {send_termination_signal(sender, 100)}; break},

        //     _ => None,

        // };

        // if result.is_some() {
        //     stop_search_vec = result.unwrap();
        // }

    }

    let _ = stop_print_sender.send(STOP_SIGNAL);
    let _ = output_thread_handle.join();

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
            
            println!("d{depth} | {best_move} | {score} | ");
        }

        let termination_signal = terminate_print_receiver.try_recv().unwrap_or(false);

        if termination_signal { return }

    }

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
        let context = rust_chess::SearchContext{board: board, receiver_channel: stop_receiver, sender_channel: info_sender};

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
            let _ = stop_sender.send(STOP_SIGNAL);
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


        let user_input = collect_user_input();

        let user_move = chess::ChessMove::from_str(&user_input).expect("Invalid move:");

        board  = board.make_move_new(user_move);
        
    }
}

fn collect_user_input() -> String{
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => {},
        Err(_) => {},
    }
    return user_input.trim().to_string();
}