use chess::{ChessMove, Square};
use log::{debug, error, info, warn};
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{sleep, JoinHandle};
use std::{io, thread, time};

use crate::search::{SearchInfo, SearchOutcome, INFINITY, MATE_THRESHOLD};
use crate::threading::SearchGroup;

const MAX_DEPTH: u8 = 64;
pub const HASH_TABLE_SIZE: u32 = 1 << 22;

const THREAD_COUNT: u8 = 1;

#[derive(Clone)]
// TODO: Change this code to use the Game Struct from chess crate
pub struct Position {
    pub board: chess::Board,
    pub hash_history: Vec<u64>,
}

struct Printer {
    str_sender: Sender<String>,
    info_sender: Sender<SearchInfo>,
    bestmove_sender: Sender<ChessMove>,
    stop_sender: Sender<bool>,

    handle: JoinHandle<()>,
}

impl Printer {
    fn print(self, text: &str) -> Printer {
        let _ = self.str_sender.send(text.to_string());
        self
    }

    fn result(self, result: SearchOutcome) -> Printer {
        let _ = self.info_sender.send((result.0, result.1, MAX_DEPTH + 1));
        self
    }

    fn bestmove(self, best_move: ChessMove) -> Printer {
        let _ = self.bestmove_sender.send(best_move);
        self
    }

    fn stop(self) {
        let _ = self.stop_sender.send(true);
        let _ = self.handle.join();
    }
}

struct PrinterReceiver {
    str: Receiver<String>,
    info: Receiver<SearchInfo>,
    bestmove: Receiver<ChessMove>,
    stop: Receiver<bool>,
}

fn build_printer() -> Printer {
    let (print_sender, print_receiver) = channel();
    let (info_sender, info_receiver) = channel();
    let (bestmove_sender, bestmove_reveicer) = channel();
    let (stop_sender, stop_receiver) = channel();

    let receiver = PrinterReceiver {
        str: print_receiver,
        info: info_receiver,
        bestmove: bestmove_reveicer,
        stop: stop_receiver,
    };

    let handle = thread::spawn(move || printing_loop(receiver));

    Printer {
        str_sender: print_sender,
        info_sender,
        bestmove_sender,
        stop_sender,
        handle,
    }
}

pub fn uci_mode() {
    info!("uci mode started\n");

    let mut position = Position {
        board: chess::Board::default(),
        hash_history: vec![],
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

        if command == "uci" {
            printer = printer.print("uciok");
        } else if command == "isready" {
            printer = printer.print("readyok");
        } else if command == "ucinewgame" {
            position = Position {
                board: chess::Board::default(),
                hash_history: vec![],
            };
        } else if command == "position" {
            position = change_position(arguments);
        } else if command == "stop" {
            if search_group.is_some() {
                let result = search_group.unwrap().stop();
                search_group = None;
                match result {
                    Ok(outcome) => {
                        info!("stop; search result: {} - {}\n", outcome.0, outcome.1);
                        printer = printer.result(outcome);
                        printer = printer.bestmove(outcome.1);
                    }
                    Err(()) => {
                        error!("stop; group.stop() failed!")
                    }
                }
            } else {
                warn!("stop; no search_group")
            }
        } else if command == "quit" {
            printer.stop();

            if search_group.is_some() {
                let result = search_group.unwrap().stop();
                match result {
                    Err(_) => {
                        error!("quit; group.stop() Error!")
                    }
                    Ok(_) => {
                        debug! {"quit; group.stop() Ok"}
                    }
                }
            } else {
                debug! {"quit; no search_group"}
            }

            info!("shutting down");
            return;
        } else if command == "go" {
            info!("start search");
            if search_group.is_none() {
                search_group = Some(SearchGroup::start(
                    position.clone(),
                    THREAD_COUNT,
                    printer.info_sender.clone(),
                    HASH_TABLE_SIZE,
                    MAX_DEPTH,
                    None,
                ));
            };
        }
    }
}

pub fn change_position(arguments: &[&str]) -> Position {
    let mut new_board = chess::Board::default();
    let mut hash_history: Vec<u64> = vec![];

    let moves_index = arguments
        .iter()
        .position(|&r| r == "moves")
        .unwrap_or(arguments.len());

    if arguments[0] == "fen" {
        let fen_string = arguments[1..moves_index].join(" ");
        new_board = chess::Board::from_str(&fen_string).unwrap_or(new_board);
    }

    hash_history.push(new_board.get_hash());

    if moves_index >= arguments.len() {
        return Position {
            board: new_board,
            hash_history,
        };
    }

    for move_str in &arguments[moves_index + 1..] {
        let parsed_move_result = chess::ChessMove::from_str(*move_str);

        if parsed_move_result.is_ok() {
            let move_obj = parsed_move_result.unwrap();

            if new_board.legal(move_obj) {
                new_board = new_board.make_move_new(move_obj);
                hash_history.push(new_board.get_hash());
            }
        }
    }

    Position {
        board: new_board,
        hash_history,
    }
}

pub fn collect_user_input() -> String {
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => {}
        Err(_) => {}
    }

    user_input.trim().to_string()
}

fn printing_loop(receiver: PrinterReceiver) {
    let update_interval = time::Duration::from_millis(10);

    loop {
        sleep(update_interval);

        let message = receiver.str.try_recv().unwrap_or("".to_string());
        if message.len() > 0 {
            println!("{message}");
        }

        let (score, mut best_move, depth) = receiver.info.try_recv().unwrap_or((
            0,
            ChessMove::new(Square::A1, Square::A1, None),
            0,
        ));

        if depth > 0 && depth <= MAX_DEPTH {
            print_info(score, best_move, depth);
        } else if depth == MAX_DEPTH + 1 {
            print_score_only(score);
            best_move = receiver.bestmove.recv().unwrap_or(best_move);
            println!("bestmove {best_move}");
        }

        let termination_signal = receiver.stop.try_recv().unwrap_or(false);
        if termination_signal {
            return;
        }
    }
}

fn print_info(score: i32, best_move: ChessMove, depth: u8) {
    if score.abs() > MATE_THRESHOLD {
        let mut mate_distance = INFINITY - score.abs();

        if score < 0 {
            mate_distance = -mate_distance;
        } else {
            mate_distance += 1
        }
        println!("info depth {depth} score mate {mate_distance} pv {best_move}")
    } else {
        println!("info depth {depth} score cp {score} pv {best_move}");
    }
}

fn print_score_only(score: i32) {
    if score.abs() > MATE_THRESHOLD {
        let mut mate_distance = INFINITY - score.abs();

        if score < 0 {
            mate_distance = -mate_distance;
        } else {
            mate_distance += 1
        }
        println!("info score mate {mate_distance}")
    } else {
        println!("info score cp {score}");
    }
}
