use crate::search::SearchDepth;
use chess::Piece;

pub const MAX_DEPTH: u8 = 64;
pub const HASH_TABLE_SIZE: u32 = 1 << 22;
pub const THREAD_COUNT: u8 = 8;
pub const BENCHMARK_THREAD_COUNT: u8 = 8;

// Evaluation constants
pub const INFINITY: i32 = 1000000;
pub const DRAW: i32 = 0;
pub const MATE_MARGIN: i32 = 100;
pub const MATE_THRESHOLD: i32 = INFINITY - MATE_MARGIN;

// Move ordering; Most Valuable Victim first, King is a dummy value for quiet moves!
pub const MVV_ORDERING: [Piece; 5] = [
    Piece::Queen,
    Piece::Rook,
    Piece::Bishop,
    Piece::Knight,
    Piece::Pawn,
];
pub const QS_ORDERING: [Piece; 5] = [
    Piece::Queen,
    Piece::Rook,
    Piece::Bishop,
    Piece::Knight,
    Piece::Pawn,
];

// Repetition detection
pub const REP_TABLE_SIZE: usize = 1 << 16;

// Search Extension
pub const MAX_EXTENSION_PLIES: SearchDepth = 3;
