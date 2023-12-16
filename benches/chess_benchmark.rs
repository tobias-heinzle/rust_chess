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

criterion_group!(benches, starting_pos_d5_benchmark);
criterion_main!(benches);