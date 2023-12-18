use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::mpsc;
use std::str::FromStr;
use rust_chess;
use chess;


fn starting_pos_d5_benchmark(c: &mut Criterion) {
    let board = chess::Board::default();
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let engine = rust_chess::ChessEngine{board: board, receiver_channel: rx, sender_channel: tx};

    c.bench_function("root_search_starting_pos_d5",
     |b| b.iter(
        || engine.root_search(black_box(5), black_box(10))));
}

fn custom_pos_d5_benchmark(c: &mut Criterion) {
    let board = chess::Board::from_str("5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - - 0 1").expect("Valid Board");
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let engine = rust_chess::ChessEngine{board: board, receiver_channel: rx, sender_channel: tx};

    c.bench_function("root_search_custom_pos_d5",
     |b| b.iter(
        || engine.root_search(black_box(5), black_box(10))));
}

fn liberman_pos_d2_benchmark(c: &mut Criterion) {
    let board = chess::Board::from_str("q2k2q1/2nqn2b/1n1P1n1b/2rnr2Q/1NQ1QN1Q/3Q3B/2RQR2B/Q2K2Q1 w - - 0 1").expect("Valid Board");
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let engine = rust_chess::ChessEngine{board: board, receiver_channel: rx, sender_channel: tx};

    c.bench_function("root_search_liberman_pos_d2",
     |b| b.iter(
        || engine.root_search(black_box(2), black_box(10))));
}

criterion_group!(benches, starting_pos_d5_benchmark, custom_pos_d5_benchmark, liberman_pos_d2_benchmark);
criterion_main!(benches);