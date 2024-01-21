use std::thread::{JoinHandle, sleep};
use std::{thread, time, io};
use std::sync::mpsc::{self, Receiver, Sender};
use std::str::FromStr;
use chess::{self, ChessMove, Square};
use log::{info, warn};

use crate::search::{SearchInfo, SearchResult, SearchContext, MATE_THRESHOLD, INFINITY};

const STOP_SIGNAL: bool = true;
const MAX_DEPTH: u8 = 64;

// TODO: Change this code to use the Game Struct from chess crate   
pub struct Position {
    pub board: chess::Board,
    pub hash_history : Vec<u64>,
}

struct SearchGroup {
    principal : SearchAgent,
    agents : Vec<SearchAgent>
}

impl SearchGroup {

    fn stop(self) -> SearchResult{

        for agent in self.agents {
            send_termination_signal(&agent.stop, 10);
            let _ = agent.handle.join();
        }

        send_termination_signal(&self.principal.stop, 10);

        let search_result = self.principal.handle.join();

        return search_result.unwrap();
    }
}

struct SearchAgent {
    stop: Sender<bool>,
    handle: JoinHandle<SearchResult>,
}

struct Printer {
    str_sender : Sender<String>,
    info_sender : Sender<SearchInfo>,
    bestmove_sender : Sender<ChessMove>,
    stop_sender : Sender<bool>,
    
    handle: JoinHandle<()>,
}

impl Printer {
    fn print(self, text: &str) -> Printer{
        let _ = self.str_sender.send(text.to_string());
        return self
    }

    fn result(self, result: SearchResult) -> Printer{
        let _ = self.info_sender.send( (result.0, result.1, MAX_DEPTH + 1) );
        return self
    }

    fn bestmove(self, best_move : ChessMove) -> Printer{
        let _ = self.bestmove_sender.send(best_move);
        return self
    }

    fn stop(self){
        send_termination_signal(&self.stop_sender, 1);
        let _ = self.handle.join();
    }
}

struct PrinterReceiver {
    str : Receiver<String>,
    info : Receiver<SearchInfo>,
    bestmove : Receiver<ChessMove>,
    stop : Receiver<bool>,
}


fn build_printer() -> Printer{
    let (print_sender, print_receiver) = mpsc::channel();
    let (info_sender, info_receiver) = mpsc::channel();
    let (bestmove_sender, bestmove_reveicer) = mpsc::channel(); 
    let (stop_sender, stop_receiver) = mpsc::channel();

    let receiver = PrinterReceiver{str : print_receiver, info : info_receiver, bestmove : bestmove_reveicer, stop : stop_receiver};
    
    let handle = thread::spawn(
        move || printing_loop(receiver));
    
    let printer = Printer{str_sender : print_sender, info_sender : info_sender,  bestmove_sender : bestmove_sender, stop_sender : stop_sender, handle};
    
    return printer;
}

fn create_search_context (info_sender: Sender<SearchInfo>, position : &Position ) -> (SearchContext, Sender<bool>) {
    let (stop_sender, stop_receiver) = mpsc::channel();
    
    let mut search_context = SearchContext::new(
        position.board, 
        stop_receiver, 
        info_sender.clone() 
    );
    for hash in position.hash_history.iter() {
        search_context.set_visited(*hash);
    }
    return (search_context, stop_sender);
}


fn start_search(num_threads: u8,  info_sender: Sender<SearchInfo>, position: &Position) -> SearchGroup {

    assert!(num_threads > 0);

    let (mut context, stop_sender) = create_search_context(info_sender, position);

    let principal = SearchAgent{
        handle: thread::spawn(move || context.root_search(MAX_DEPTH)), 
        stop: stop_sender
    };
    
    let mut agents:  Vec<SearchAgent> = vec![];

    let (dummy_sender, _) = mpsc::channel();

    for _ in 0 .. num_threads - 1 {
        let (mut agent_context, agent_stop_sender) = create_search_context(dummy_sender.clone(), position);
        let agent = SearchAgent{
            handle : thread::spawn(move || agent_context.root_search(MAX_DEPTH)), 
            stop : agent_stop_sender
        };

        agents.push(agent);

    };

    let search_group = SearchGroup {
        principal : principal,
        agents : agents
    };

    return search_group;
}


pub fn uci_mode(){
    info!("uci mode started\n");

    let num_threads = 1;

    let mut position = Position{
        board : chess::Board::default(),
        hash_history : vec![],
    };


    let mut printer = build_printer();
    let mut search_group: Option<SearchGroup> = None;

    loop {
        let input_line = collect_user_input();
        info!("{}\n", input_line);

        let input: Vec<&str> = input_line.split(" ").collect();
        let command = input[0];
        let arguments = &input[1..];
        info!("{}\n", command);

        if      command == "uci"        { printer = printer.print("uciok"); }
        else if command == "isready"    { printer = printer.print("readyok"); }
        else if command == "ucinewgame" { position = Position{ board : chess::Board::default(), hash_history : vec![] }; }
        else if command == "position"   { position = change_position(arguments); }
        else if command == "stop"       { 
            if search_group.is_some(){
                let result = search_group.unwrap().stop(); 
                search_group = None;
                info!("search result: {} - {}\n", result.0, result.1);
                printer = printer.result(result);
                printer = printer.bestmove(result.1);

               
            }
        }
        else if command == "quit"       { 
            printer.stop();

            if search_group.is_some() {
                let _ = search_group.unwrap().stop();
            }; 
            info!("shutting down");
            return
        }
        else if command == "go"         {
            info!("start search");
            if search_group.is_none() {
                search_group = Some(start_search(num_threads, printer.info_sender.clone(), &position));
            }; 
        }

    }

}


fn send_termination_signal(sender: &Sender<bool>, n_signals: i32) {
    for _ in 0 .. n_signals { 
        let _ = sender.send(STOP_SIGNAL); 
    }
}

pub fn change_position(arguments: &[&str]) -> Position{
    let mut new_board = chess::Board::default();
    let mut hash_history: Vec<u64> = vec![];

    let moves_index = arguments.iter().position(|&r| r == "moves").unwrap_or(arguments.len());

    if arguments[0] == "fen" {
        let fen_string = arguments[1 .. moves_index].join(" ");
        new_board = chess::Board::from_str(&fen_string).unwrap_or(new_board);
    }

    hash_history.push(new_board.get_hash());
    
    if moves_index >= arguments.len() {
        return Position {
            board : new_board,
            hash_history : hash_history
        }; 
    }

    for move_str in &arguments[moves_index + 1 .. ]{

        let parsed_move_result = chess::ChessMove::from_str(*move_str);

        if parsed_move_result.is_ok() {
            let move_obj = parsed_move_result.unwrap();

            if new_board.legal(move_obj){
                new_board = new_board.make_move_new(move_obj);
                hash_history.push(new_board.get_hash());
            }
        }
        
        
    }

    return Position {
        board : new_board,
        hash_history : hash_history
    }; 
}


pub fn collect_user_input() -> String{
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => {},
        Err(_) => {},
    }
    return user_input.trim().to_string();
}

fn printing_loop(receiver: PrinterReceiver){
    let update_interval = time::Duration::from_millis(10);

    loop {
        sleep(update_interval);

        let message = receiver.str.try_recv().unwrap_or("".to_string());
        if message.len() > 0 { println!("{message}"); }

        let (score, mut best_move, depth) = receiver.info.try_recv().unwrap_or(
            (0, ChessMove::new(Square::A1, Square::A1, None), 0)
        );
        
        if depth > 0 && depth <= MAX_DEPTH {    
            print_info(score, best_move, depth);
        }
        else if depth == MAX_DEPTH + 1{
            print_score_only(score);
            best_move = receiver.bestmove.recv().unwrap_or(best_move);
            println!("bestmove {best_move}");
        }

        let termination_signal = receiver.stop.try_recv().unwrap_or(false);
        if termination_signal { return }

    }

}

fn print_info(score: i32, best_move: ChessMove, depth: u8) {
    if score.abs() > MATE_THRESHOLD{
        let mut mate_distance = INFINITY - score.abs();

        if score < 0 {
            mate_distance = -mate_distance;
        }
        else {
            mate_distance += 1
        }
        println!("info depth {depth} score mate {mate_distance} pv {best_move}")
    }
    else {
        println!("info depth {depth} score cp {score} pv {best_move}");
    }
}

fn print_score_only(score: i32){
    if score.abs() > MATE_THRESHOLD{
        let mut mate_distance = INFINITY - score.abs();

        if score < 0 {
            mate_distance = -mate_distance;
        }
        else {
            mate_distance += 1
        }
        println!("info score mate {mate_distance}")
    }
    else {
        println!("info score cp {score}");
    }
}