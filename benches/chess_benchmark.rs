use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::mpsc;
use std::str::FromStr;
use std::task::Context;
use rust_chess;
use chess;


fn starting_pos_d5_benchmark(c: &mut Criterion) {
    let board = chess::Board::default();
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let context = rust_chess::SearchContext{board: board, receiver_channel: rx, sender_channel: tx};

    c.bench_function("root_search_starting_pos_d5",
     |b| b.iter(
        || context.root_search(black_box(5))));
}

fn custom_pos_d5_benchmark(c: &mut Criterion) {
    let board = chess::Board::from_str("5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - - 0 1").expect("Valid Board");
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let context = rust_chess::SearchContext{board: board, receiver_channel: rx, sender_channel: tx};

    c.bench_function("root_search_custom_pos_d5",
     |b| b.iter(
        || context.root_search(black_box(5))));
}

fn chezzz_pos_d3_benchmark(c: &mut Criterion) {
    let board = chess::Board::from_str("r4rk1/1p1n1pp1/1bq1bn1p/p1pp4/2P2B2/1NNP2P1/PPQ2PBP/R4RK1 w - - 0 19").expect("Valid Board");
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let context = rust_chess::SearchContext{board: board, receiver_channel: rx, sender_channel: tx};

    c.bench_function("root_search_chezzz_pos_d2",
     |b| b.iter(
        || context.root_search(black_box(3))));
}

fn liberman_pos_d1_benchmark(c: &mut Criterion) {
    let board = chess::Board::from_str("q2k2q1/2nqn2b/1n1P1n1b/2rnr2Q/1NQ1QN1Q/3Q3B/2RQR2B/Q2K2Q1 w - - 0 1").expect("Valid Board");
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let context = rust_chess::SearchContext{board: board, receiver_channel: rx, sender_channel: tx};

    c.bench_function("root_search_liberman_pos_d1",
     |b| b.iter(
        || context.root_search(black_box(1))));
}

criterion_group!(benches, starting_pos_d5_benchmark, custom_pos_d5_benchmark, chezzz_pos_d3_benchmark, liberman_pos_d1_benchmark);
criterion_main!(benches);