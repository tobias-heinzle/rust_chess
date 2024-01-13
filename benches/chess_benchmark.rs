use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::mpsc;
use std::str::FromStr;
use rust_chess;
use chess;


fn starting_d5(c: &mut Criterion) {
    let board = chess::Board::default();
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let mut context = rust_chess::search::SearchContext::new(board, rx, tx);

    c.bench_function("root_search_starting_d5",
     |b| b.iter(
        || context.root_search(black_box(5))));
}
fn starting_d7(c: &mut Criterion) {
    let board = chess::Board::default();
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let mut context = rust_chess::search::SearchContext::new(board, rx, tx);

    c.bench_function("root_search_starting_d7",
     |b| b.iter(
        || context.root_search(black_box(7))));
}

fn custom_d5(c: &mut Criterion) {
    let board = chess::Board::from_str("5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - - 0 1").expect("Valid Board");
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let mut context = rust_chess::search::SearchContext::new(board, rx, tx);

    c.bench_function("root_search_custom_d5",
     |b| b.iter(
        || context.root_search(black_box(5))));
}

fn chezzz_d3(c: &mut Criterion) {
    let board = chess::Board::from_str("r4rk1/1p1n1pp1/1bq1bn1p/p1pp4/2P2B2/1NNP2P1/PPQ2PBP/R4RK1 w - - 0 19").expect("Valid Board");
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let mut context = rust_chess::search::SearchContext::new(board, rx, tx);

    c.bench_function("root_search_chezzz_d2",
     |b| b.iter(
        || context.root_search(black_box(3))));
}

fn liberman_d1(c: &mut Criterion) {
    let board = chess::Board::from_str("q2k2q1/2nqn2b/1n1P1n1b/2rnr2Q/1NQ1QN1Q/3Q3B/2RQR2B/Q2K2Q1 w - - 0 1").expect("Valid Board");
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let mut context = rust_chess::search::SearchContext::new(board, rx, tx);

    c.bench_function("root_search_liberman_d1",
     |b| b.iter(
        || context.root_search(black_box(1))));
}

fn middlegame_d6(c: &mut Criterion) {
    let board = chess::Board::from_str("r4r1k/1pq1p1bp/1pnp2p1/p2B4/2PP2Q1/4B2P/PP3PP1/1R3RK1 w - - 6 20").expect("Valid Board");
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let mut context = rust_chess::search::SearchContext::new(board, rx, tx);

    c.bench_function("root_search_middlegame_d6",
     |b| b.iter(
        || context.root_search(black_box(6))));
}

fn endgame_d10(c: &mut Criterion) {
    let board = chess::Board::from_str("8/p7/3n2k1/4K1P1/1P6/6N1/P6p/8 b - - 3 51").expect("Valid Board");
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let mut context = rust_chess::search::SearchContext::new(board, rx, tx);

    c.bench_function("root_endgame_d10",
     |b| b.iter(
        || context.root_search(black_box(6))));
}

fn mate_in_7_d9(c: &mut Criterion) {
    let board = chess::Board::from_str("r6k/ppp4b/8/3p3Q/3q3R/1P4P1/P5PP/6K1 w - - 0 41").expect("Valid Board");
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let mut context = rust_chess::search::SearchContext::new(board, rx, tx);

    c.bench_function("root_mate_in_7_d9",
     |b| b.iter(
        || context.root_search(black_box(9))));
}


criterion_group!(benches, 
    starting_d5, 
    starting_d7,
    endgame_d10,
    mate_in_7_d9,
    custom_d5, 
    middlegame_d6,
    chezzz_d3, 
    liberman_d1,
    );
criterion_main!(benches);