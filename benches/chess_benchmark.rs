use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_chess::search::SearchContext;
use std::sync::mpsc;
use std::str::FromStr;
use rust_chess;
use chess;


fn setup_context(board: chess::Board) -> SearchContext {
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    return rust_chess::search::SearchContext::new(board, rx, tx);
}

fn startpos_d5(c: &mut Criterion) {
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

fn startpos_d7(c: &mut Criterion) {
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

fn custom_d5(c: &mut Criterion) {
    let board = chess::Board::from_str("5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - - 0 1").expect("Valid Board");

    c.bench_function("custom_d5",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(5))
            }
        )
    );
}

fn chezzz_d3(c: &mut Criterion) {
    let board = chess::Board::from_str("r4rk1/1p1n1pp1/1bq1bn1p/p1pp4/2P2B2/1NNP2P1/PPQ2PBP/R4RK1 w - - 0 19").expect("Valid Board");

    c.bench_function("chezzz_d3",
    |b| b.iter(
        || {
                let mut context = setup_context(board.clone());
                context.root_search(black_box(3))
            }
        )
    );
}

fn liberman_d1(c: &mut Criterion) {
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

fn middlegame_d6(c: &mut Criterion) {
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

fn endgame_d10(c: &mut Criterion) {
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

fn mate_in_7_d7(c: &mut Criterion) {
    let board = chess::Board::from_str("r6k/ppp4b/8/3p3Q/3q3R/1P4P1/P5PP/6K1 w - - 0 41").expect("Valid Board");

    c.bench_function(
        "mate_in_7_d7",
        |b| b.iter(
         || {
            let mut context = setup_context(board.clone());
            context.root_search(black_box(7))}
        
        )
    );
}


criterion_group!(benches, 
    startpos_d5, 
    startpos_d7,
    endgame_d10,
    mate_in_7_d7,
    custom_d5, 
    middlegame_d6,
    chezzz_d3, 
    liberman_d1,
    );
criterion_main!(benches);