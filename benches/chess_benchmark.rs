use chess;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_chess;
use std::str::FromStr;
use std::sync::mpsc;

use rust_chess::config;
use rust_chess::test_utils::setup_test_context;
use rust_chess::threading::SearchGroup;
use rust_chess::uci::Position;

fn startpos(c: &mut Criterion) {
    let board = chess::Board::default();

    c.bench_function("startpos_d4", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(4))
        })
    });
}

fn startpos_1(c: &mut Criterion) {
    let board = chess::Board::default();

    c.bench_function("startpos_d5", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(5))
        })
    });
}

fn startpos_2(c: &mut Criterion) {
    let board = chess::Board::default();

    c.bench_function("startpos_d6", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(6))
        })
    });
}

fn startpos_3_single(c: &mut Criterion) {
    let board = chess::Board::default();

    c.bench_function("startpos_d7_single", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(7))
        })
    });
}

fn startpos_3_parallel(c: &mut Criterion) {
    let board = chess::Board::default();

    // TODO: change maybe move order to exploit table

    let position = Position {
        board: board,
        hash_history: vec![],
    };
    let (info_sender, _) = mpsc::channel();

    c.bench_function("startpos_d7_parallel", |b| {
        b.iter(|| {
            let search_group = SearchGroup::start(
                position.clone(),
                config::BENCHMARK_THREAD_COUNT,
                info_sender.clone(),
                config::HASH_TABLE_SIZE,
                black_box(7),
                None,
            );
            let _ = search_group.await_principal();
        })
    });
}

fn custom(c: &mut Criterion) {
    let board = chess::Board::from_str("5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - - 0 1")
        .expect("Valid Board");

    c.bench_function("custom_d4", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(4))
        })
    });
}

fn chezzz(c: &mut Criterion) {
    let board = chess::Board::from_str(
        "r4rk1/1p1n1pp1/1bq1bn1p/p1pp4/2P2B2/1NNP2P1/PPQ2PBP/R4RK1 w - - 0 19",
    )
    .expect("Valid Board");

    c.bench_function("chezzz_d2", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(2))
        })
    });
}

fn mate_in_3_single(c: &mut Criterion) {
    let board = chess::Board::from_str("3r4/pR2N3/2pkb3/5p2/8/2B5/qP3PPP/4R1K1 w - - 1 1")
        .expect("Valid Board");

    c.bench_function("mate_in_three_single", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(6))
        })
    });
}

fn mate_in_3_parallel(c: &mut Criterion) {
    let board = chess::Board::from_str("3r4/pR2N3/2pkb3/5p2/8/2B5/qP3PPP/4R1K1 w - - 1 1")
        .expect("Valid Board");

    // TODO: change maybe move order to exploit table

    let position = Position {
        board: board,
        hash_history: vec![],
    };
    let (info_sender, _) = mpsc::channel();

    c.bench_function("mate_in_three_parallel", |b| {
        b.iter(|| {
            let search_group = SearchGroup::start(
                position.clone(),
                config::BENCHMARK_THREAD_COUNT,
                info_sender.clone(),
                config::HASH_TABLE_SIZE,
                black_box(6),
                None,
            );
            let _ = search_group.await_principal();
        })
    });
}

fn liberman(c: &mut Criterion) {
    let board = chess::Board::from_str(
        "q2k2q1/2nqn2b/1n1P1n1b/2rnr2Q/1NQ1QN1Q/3Q3B/2RQR2B/Q2K2Q1 w - - 0 1",
    )
    .expect("Valid Board");

    c.bench_function("liberman_d1", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(1))
        })
    });
}

fn middlegame(c: &mut Criterion) {
    let board =
        chess::Board::from_str("r4r1k/1pq1p1bp/1pnp2p1/p2B4/2PP2Q1/4B2P/PP3PP1/1R3RK1 w - - 6 20")
            .expect("Valid Board");

    c.bench_function("middlegame_d5", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(5))
        })
    });
}

fn middlegame_1(c: &mut Criterion) {
    let board =
        chess::Board::from_str("r4r1k/1pq1p1bp/1pnp2p1/p2B4/2PP2Q1/4B2P/PP3PP1/1R3RK1 w - - 6 20")
            .expect("Valid Board");

    c.bench_function("middlegame_d6", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(6))
        })
    });
}

fn middlegame_2_single(c: &mut Criterion) {
    let board =
        chess::Board::from_str("r4r1k/1pq1p1bp/1pnp2p1/p2B4/2PP2Q1/4B2P/PP3PP1/1R3RK1 w - - 6 20")
            .expect("Valid Board");

    c.bench_function("middlegame_d7_single", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(7))
        })
    });
}

fn middlegame_2_parallel(c: &mut Criterion) {
    let board =
        chess::Board::from_str("r4r1k/1pq1p1bp/1pnp2p1/p2B4/2PP2Q1/4B2P/PP3PP1/1R3RK1 w - - 6 20")
            .expect("Valid Board");

    let position = Position {
        board: board,
        hash_history: vec![],
    };
    let (info_sender, _) = mpsc::channel();

    c.bench_function("middlegame_d7_parallel", |b| {
        b.iter(|| {
            let search_group = SearchGroup::start(
                position.clone(),
                config::BENCHMARK_THREAD_COUNT,
                info_sender.clone(),
                config::HASH_TABLE_SIZE,
                black_box(7),
                None,
            );
            let _ = search_group.await_principal();
        })
    });
}

fn middlegame_3_single(c: &mut Criterion) {
    let board =
        chess::Board::from_str("2kr2nr/pp3ppp/1q2p1b1/2b5/Q1PN4/4B3/PP2NPPP/R4RK1 b - - 4 14")
            .expect("Valid Board");

    c.bench_function("middlegame_3_d7_single", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(7))
        })
    });
}

fn middlegame_3_parallel(c: &mut Criterion) {
    let board =
        chess::Board::from_str("2kr2nr/pp3ppp/1q2p1b1/2b5/Q1PN4/4B3/PP2NPPP/R4RK1 b - - 4 14")
            .expect("Valid Board");

    let position = Position {
        board: board,
        hash_history: vec![],
    };
    let (info_sender, _) = mpsc::channel();

    c.bench_function("middlegame_3_d7_parallel", |b| {
        b.iter(|| {
            let search_group = SearchGroup::start(
                position.clone(),
                config::BENCHMARK_THREAD_COUNT,
                info_sender.clone(),
                config::HASH_TABLE_SIZE,
                black_box(7),
                None,
            );
            let _ = search_group.await_principal();
        })
    });
}

fn endgame(c: &mut Criterion) {
    let board =
        chess::Board::from_str("8/p7/3n2k1/4K1P1/1P6/6N1/P6p/8 b - - 3 51").expect("Valid Board");

    c.bench_function("endgame_d8", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(8))
        })
    });
}

fn endgame_1(c: &mut Criterion) {
    let board =
        chess::Board::from_str("8/p7/3n2k1/4K1P1/1P6/6N1/P6p/8 b - - 3 51").expect("Valid Board");

    c.bench_function("endgame_d10", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(10))
        })
    });
}

fn endgame_2_single(c: &mut Criterion) {
    let board =
        chess::Board::from_str("8/p7/3n2k1/4K1P1/1P6/6N1/P6p/8 b - - 3 51").expect("Valid Board");

    c.bench_function("endgame_d11_single", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(11))
        })
    });
}

fn endgame_2_parallel(c: &mut Criterion) {
    let board =
        chess::Board::from_str("8/p7/3n2k1/4K1P1/1P6/6N1/P6p/8 b - - 3 51").expect("Valid Board");

    let position = Position {
        board: board,
        hash_history: vec![],
    };
    let (info_sender, _) = mpsc::channel();

    c.bench_function("endgame_d11_parallel", |b| {
        b.iter(|| {
            let search_group = SearchGroup::start(
                position.clone(),
                config::BENCHMARK_THREAD_COUNT,
                info_sender.clone(),
                config::HASH_TABLE_SIZE,
                black_box(11),
                None,
            );
            let _ = search_group.await_principal();
        })
    });
}

fn mate_in_7(c: &mut Criterion) {
    let board = chess::Board::from_str("r6k/ppp4b/8/3p3Q/3q3R/1P4P1/P5PP/6K1 w - - 0 41")
        .expect("Valid Board");

    c.bench_function("mate_in_7_d5", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(5))
        })
    });
}

fn stalemate(c: &mut Criterion) {
    let board =
        chess::Board::from_str("2n1bK1k/2n5/3pp1p1/5Pp1/7r/8/7R/8 w - - 2 8").expect("Valid Board");

    c.bench_function("stalemate_d7", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(7))
        })
    });
}

fn out_of_opening_single(c: &mut Criterion) {
    let board = chess::Board::from_str(
        "r4rk1/1ppqbppp/p1npn1b1/P3p3/4P3/2PPNN1P/1PB2PP1/R1BQR1K1 b - - 0 15",
    )
    .expect("Valid Board");

    c.bench_function("out_of_opening_d7_single", |b| {
        b.iter(|| {
            let mut context = setup_test_context(board.clone());
            context.root_search(black_box(7))
        })
    });
}

fn out_of_opening_parallel(c: &mut Criterion) {
    let board = chess::Board::from_str(
        "r4rk1/1ppqbppp/p1npn1b1/P3p3/4P3/2PPNN1P/1PB2PP1/R1BQR1K1 b - - 0 15",
    )
    .expect("Valid Board");

    let position = Position {
        board: board,
        hash_history: vec![],
    };
    let (info_sender, _) = mpsc::channel();

    c.bench_function("out_of_opening_d7_parallel", |b| {
        b.iter(|| {
            let search_group = SearchGroup::start(
                position.clone(),
                config::BENCHMARK_THREAD_COUNT,
                info_sender.clone(),
                config::HASH_TABLE_SIZE,
                black_box(7),
                None,
            );
            let _ = search_group.await_principal();
        })
    });
}

criterion_group!(
    benches,
    out_of_opening_single,
    out_of_opening_parallel,
    middlegame_3_single,
    middlegame_3_parallel,
    endgame,
    endgame_1,
    endgame_2_single,
    endgame_2_parallel,
    mate_in_7,
    custom,
    middlegame,
    middlegame_1,
    middlegame_2_single,
    middlegame_2_parallel,
    startpos,
    startpos_1,
    startpos_2,
    startpos_3_single,
    startpos_3_parallel,
    mate_in_3_single,
    mate_in_3_parallel,
    stalemate,
    chezzz,
    liberman,
);
criterion_main!(benches);
