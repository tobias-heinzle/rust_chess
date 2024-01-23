use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_chess::search::SearchContext;
use std::sync::{Arc, mpsc};
use std::str::FromStr;
use std::thread;
use rust_chess;
use chess;

use chess::{ChessMove, Square};

use rust_chess::table::{TranspositionTable, TableEntryData, TableReference, ScoreBound};
use rust_chess::uci::{SearchAgent, SearchGroup, Position, create_search_context, HASH_TABLE_SIZE};

fn setup_context(board: chess::Board) -> SearchContext {
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let hash_table = Arc::new(
        rust_chess::table::TranspositionTable::new(
            rust_chess::uci::HASH_TABLE_SIZE, 
            rust_chess::table::TableEntryData{
                best_move : chess::ChessMove::new(
                    chess::Square::A1, 
                    chess::Square::A1, 
                    None), 
                score : 0, 
                depth : 0, 
                score_bound : rust_chess::table::ScoreBound::LowerBound}
            )
        );
    
    return rust_chess::search::SearchContext::new(board, rx, tx, Arc::clone(&hash_table));
}

fn startpos(c: &mut Criterion) {
    let board = chess::Board::default();

    c.bench_function("startpos_d4",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(4))
            }
        )
    );
}

fn startpos_1(c: &mut Criterion) {
    let board = chess::Board::default();

    c.bench_function("startpos_d5",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(5))
            }
        )
    );
}

fn startpos_2(c: &mut Criterion) {
    let board = chess::Board::default();

    c.bench_function("startpos_d6",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(6))
            }
        )
    );
}

fn startpos_3(c: &mut Criterion) {
    let board = chess::Board::default();

    c.bench_function("startpos_d7",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(7))
            }
        )
    );
}

fn startpos_3_parallel(c: &mut Criterion) {
    let board = chess::Board::default();

    let num_threads = 2;

    let mut position = Position{
        board : board,
        hash_history : vec![],
    };

    let hash_table = TableReference::new(TranspositionTable::new(HASH_TABLE_SIZE, TableEntryData{best_move : ChessMove::new(Square::A1, Square::A1, None), score : 0, depth : 0, score_bound : ScoreBound::LowerBound}));
    
    let (tx, _) = mpsc::channel();
    let (mut context, stop_sender) = create_search_context(tx, &position, hash_table.clone());

    let principal = SearchAgent{
        handle: thread::spawn(move || context.root_search(7)), 
        stop: stop_sender
    };
    
    let mut agents:  Vec<SearchAgent> = vec![];

    let (dummy_sender, _) = mpsc::channel();

    for _ in 0 .. num_threads - 1 {
        let (mut agent_context, agent_stop_sender) = create_search_context(dummy_sender.clone(), &position, hash_table.clone());
        let agent = SearchAgent{
            handle : thread::spawn(move || agent_context.root_search(7)), 
            stop : agent_stop_sender
        };

        agents.push(agent);

    };

    let search_group = SearchGroup {
        principal : principal,
        agents : agents
    };

    c.bench_function("startpos_d7_parallel",
    |b| b.iter(
        || {
            let board = chess::Board::default();

            let num_threads = 4;
        
            let mut position = Position{
                board : board,
                hash_history : vec![],
            };
        
            let hash_table = TableReference::new(TranspositionTable::new(HASH_TABLE_SIZE, TableEntryData{best_move : ChessMove::new(Square::A1, Square::A1, None), score : 0, depth : 0, score_bound : ScoreBound::LowerBound}));
            
            let (tx, _) = mpsc::channel();
            let (mut context, stop_sender) = create_search_context(tx, &position, hash_table.clone());
        
            let principal = SearchAgent{
                handle: thread::spawn(move || context.root_search(7)), 
                stop: stop_sender
            };
            
            let mut agents:  Vec<SearchAgent> = vec![];
        
            let (dummy_sender, _) = mpsc::channel();
        
            for _ in 0 .. num_threads - 1 {
                let (mut agent_context, agent_stop_sender) = create_search_context(dummy_sender.clone(), &position, hash_table.clone());
                let agent = SearchAgent{
                    handle : thread::spawn(move || agent_context.root_search(7)), 
                    stop : agent_stop_sender
                };
        
                agents.push(agent);
        
            };
        
            let search_group = SearchGroup {
                principal : principal,
                agents : agents
            };

            search_group.wait();
            }
        )
    );
}

fn custom(c: &mut Criterion) {
    let board = chess::Board::from_str("5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - - 0 1").expect("Valid Board");

    c.bench_function("custom_d4",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(4))
            }
        )
    );
}

fn chezzz(c: &mut Criterion) {
    let board = chess::Board::from_str("r4rk1/1p1n1pp1/1bq1bn1p/p1pp4/2P2B2/1NNP2P1/PPQ2PBP/R4RK1 w - - 0 19").expect("Valid Board");

    c.bench_function("chezzz_d2",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(2))
            }
        )
    );
}

fn mate_in_3(c: &mut Criterion) {
    let board = chess::Board::from_str("3r4/pR2N3/2pkb3/5p2/8/2B5/qP3PPP/4R1K1 w - - 1 1").expect("Valid Board");

    c.bench_function("mate_in_three",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(6))
            }
        )
    );
}

fn liberman(c: &mut Criterion) {
    let board = chess::Board::from_str("q2k2q1/2nqn2b/1n1P1n1b/2rnr2Q/1NQ1QN1Q/3Q3B/2RQR2B/Q2K2Q1 w - - 0 1").expect("Valid Board");

    c.bench_function("liberman_d1",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(1))
            }
        )
    );
}

fn middlegame(c: &mut Criterion) {
    let board = chess::Board::from_str("r4r1k/1pq1p1bp/1pnp2p1/p2B4/2PP2Q1/4B2P/PP3PP1/1R3RK1 w - - 6 20").expect("Valid Board");

    c.bench_function("middlegame_d5",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(5))
            }
        )
    );
}


fn middlegame_1(c: &mut Criterion) {
    let board = chess::Board::from_str("r4r1k/1pq1p1bp/1pnp2p1/p2B4/2PP2Q1/4B2P/PP3PP1/1R3RK1 w - - 6 20").expect("Valid Board");

    c.bench_function("middlegame_d6",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(6))
            }
        )
    );
}

fn middlegame_2(c: &mut Criterion) {
    let board = chess::Board::from_str("r4r1k/1pq1p1bp/1pnp2p1/p2B4/2PP2Q1/4B2P/PP3PP1/1R3RK1 w - - 6 20").expect("Valid Board");

    c.bench_function("middlegame_d7",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(7))
            }
        )
    );
}


fn endgame(c: &mut Criterion) {
    let board = chess::Board::from_str("8/p7/3n2k1/4K1P1/1P6/6N1/P6p/8 b - - 3 51").expect("Valid Board");
    
    c.bench_function("endgame_d8",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(8))
            }
        )
    );
}


fn endgame_1(c: &mut Criterion) {
    let board = chess::Board::from_str("8/p7/3n2k1/4K1P1/1P6/6N1/P6p/8 b - - 3 51").expect("Valid Board");
    
    c.bench_function("endgame_d10",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(10))
            }
        )
    );
}

fn endgame_2(c: &mut Criterion) {
    let board = chess::Board::from_str("8/p7/3n2k1/4K1P1/1P6/6N1/P6p/8 b - - 3 51").expect("Valid Board");
    
    c.bench_function("endgame_d11",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(11))
            }
        )
    );
}

fn mate_in_7(c: &mut Criterion) {
    let board = chess::Board::from_str("r6k/ppp4b/8/3p3Q/3q3R/1P4P1/P5PP/6K1 w - - 0 41").expect("Valid Board");

    c.bench_function(
        "mate_in_7_d5",
        |b| b.iter(
         || {
            let mut context = setup_context(board.clone());
            context.root_search(black_box(5))}
        
        )
    );
}

fn stalemate(c: &mut Criterion) {
    let board = chess::Board::from_str("2n1bK1k/2n5/3pp1p1/5Pp1/7r/8/7R/8 w - - 2 8").expect("Valid Board");

    c.bench_function(
        "stalemate_d7",
        |b| b.iter(
         || {
            let mut context = setup_context(board.clone());
            context.root_search(black_box(7))}
        
        )
    );
}

criterion_group!(benches, 
    startpos, 
    startpos_1,
    startpos_2,
    startpos_3,
    startpos_3_parallel,
    mate_in_3,
    endgame,
    endgame_1,
    endgame_2,
    mate_in_7,
    custom, 
    middlegame,
    middlegame_1,
    middlegame_2,
    stalemate,
    chezzz, 
    liberman,
    );
criterion_main!(benches);