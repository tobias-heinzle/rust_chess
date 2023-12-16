use criterion::{black_box, criterion_group, criterion_main, Criterion};

use std::str::FromStr;
use rust_chess::{self, root_search};
use chess;


fn starting_pos_d5_benchmark(c: &mut Criterion) {
    let board = chess::Board::default();

    c.bench_function("root_search_starting_pos_d5",
     |b| b.iter(
        || root_search(black_box(&board), black_box(5))));
}

fn custom_pos_d5_benchmark(c: &mut Criterion) {
    let board = chess::Board::from_str("5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - - 0 1").expect("Valid Board");

    c.bench_function("root_search_custom_pos_d5",
     |b| b.iter(
        || root_search(black_box(&board), black_box(5))));
}

criterion_group!(benches, starting_pos_d5_benchmark, custom_pos_d5_benchmark);
criterion_main!(benches);