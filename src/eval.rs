use crate::search::PositionScore;
use chess::Piece::{Bishop, Knight, Pawn, Queen, Rook};
use chess::{BitBoard, Board, Color, Square, EMPTY};

const PAWN_VALUE: i32 = 80;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 310;
const ROOK_VALUE: i32 = 450;
const QUEEN_VALUE: i32 = 900;

const PIN_VALUE: i32 = 5;
const MOBILITY_VALUE: i32 = 1;
const BISHOP_MOBILITY_VALUE: i32 = 2;
const IN_CHECK_PENALTY: i32 = 30;

const BLACK_HALF: BitBoard = BitBoard(0xffffffff00000000);
const WHITE_HALF: BitBoard = BitBoard(0xffffffff);

const CONNECTED_PAWN_VALUE: i32 = 2;

const ATTACK_VALUE: i32 = 2;

const INVASION_BONUS: i32 = 3;

const SIDE_TO_MOVE_BONUS: i32 = 1;

const MAX_MATERIAL: i32 = 78;

macro_rules! add_value {
    ( $table:expr, $value:expr ) => {{
        let mut table = $table;
        let mut i = 0;
        while i < 64 {
            table[i] += $value;
            i += 1
        }
        table
    }};
}

#[rustfmt::skip]
const PAWN_SHIELD_VALUE_TABLE: [i32; 64] = [
    10, 10, 10,  0,  0,  0, 10, 10,
     5,  5,  0,  0,  0,  0,  5,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,
     5,  5,  0,  0,  0,  0,  5,  5,
    10, 10, 10,  0,  0,  0, 10, 10,
];
#[rustfmt::skip]
const PAWN_TABLE_OPENING_WHITE: [i32; 64] = add_value!([
    0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  3,  3,  0,  0,  0, 
    0,  0,  3, 10, 10,  3,  0,  0, 
    0,  0,  3, 10, 10,  3,  0,  0, 
    0,  0,  0,  3,  3,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0,
], PAWN_VALUE);
#[rustfmt::skip]
const PAWN_TABLE_OPENING_BLACK: [i32; 64] = add_value!([
     0,  0,  0,  0,  0,  0,  0,  0, 
     0,  0,  0,  0,  0,  0,  0,  0, 
     0,  0,  0,  3,  3,  0,  0,  0, 
     0,  0,  3, 10, 10,  3,  0,  0, 
     0,  0,  3, 10, 10,  3,  0,  0, 
     0,  0,  0,  3,  3,  0,  0,  0, 
     0,  0,  0,  0,  0,  0,  0,  0, 
     0,  0,  0,  0,  0,  0,  0,  0,
], PAWN_VALUE);
#[rustfmt::skip]
const PAWN_TABLE_ENDGAME_WHITE: [i32; 64] = add_value!([
    10, 10, 10, 10, 10, 10, 10, 10, 
    10, 10, 10, 10, 10, 10, 10, 10, 
    11, 11, 11, 11, 11, 11, 11, 11, 
    13, 13, 13, 13, 13, 13, 13, 13,  
    16, 16, 16, 16, 16, 16, 16, 16,  
    20, 20, 20, 20, 20, 20, 20, 20, 
    25, 25, 25, 25, 25, 25, 25, 25, 
    25, 25, 25, 25, 25, 25, 25, 25,
], PAWN_VALUE);
#[rustfmt::skip]
const PAWN_TABLE_ENDGAME_BLACK: [i32; 64] = add_value!([
    25, 25, 25, 25, 25, 25, 25, 25,
    25, 25, 25, 25, 25, 25, 25, 25, 
    20, 20, 20, 20, 20, 20, 20, 20, 
    16, 16, 16, 16, 16, 16, 16, 16,  
    13, 13, 13, 13, 13, 13, 13, 13,  
    11, 11, 11, 11, 11, 11, 11, 11, 
    10, 10, 10, 10, 10, 10, 10, 10, 
    10, 10, 10, 10, 10, 10, 10, 10, 
], PAWN_VALUE);
#[rustfmt::skip]
const KNIGHT_TABLE: [i32; 64] = add_value!([
    2,  3,  4,  4,  4,  4,  3,  2, 
    3,  4,  6,  6,  6,  6,  4,  3, 
    4,  6,  8,  8,  8,  8,  6,  4, 
    4,  6,  8,  8,  8,  8,  6,  4, 
    4,  6,  8,  8,  8,  8,  6,  4, 
    4,  6,  8,  8,  8,  8,  6,  4, 
    3,  4,  6,  6,  6,  6,  4,  3,
    2,  3,  4,  4,  4,  4,  3,  2,
], KNIGHT_VALUE);
#[rustfmt::skip]
const ROOK_TABLE_WHITE: [i32; 64] = add_value!([
    0,  0,  0,  1,  0,  1,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0, 
    2,  2,  2,  2,  2,  2,  2,  2, 
    0,  0,  0,  0,  0,  0,  0,  0,
], ROOK_VALUE);
#[rustfmt::skip]
const ROOK_TABLE_BLACK: [i32; 64] = add_value!([
    0,  0,  0,  0,  0,  0,  0,  0, 
    2,  2,  2,  2,  2,  2,  2,  2, 
    0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  1,  0,  1,  0,  0, 
], ROOK_VALUE);
#[rustfmt::skip]
const KING_TABLE_ENDGAME: [i32; 64] = [
    -5, -5, -5, -5, -5, -5, -5, -5, 
    -5,  0,  0,  0,  0,  0,  0, -5, 
    -5,  0,  3,  3,  3,  3,  0, -5, 
    -5,  0,  3,  4,  4,  3,  0, -5, 
    -5,  0,  3,  4,  4,  3,  0, -5, 
    -5,  0,  3,  3,  3,  3,  0, -5, 
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5, -5, -5, -5, -5, -5, -5, -5,
];

#[inline]
pub fn evaluate(board: &Board) -> PositionScore {
    // TODO: Undefended pieces, open files for rooks, past pawns , outposts
    //       Draw by insufficient material (no pawns and total material <= bishop)

    let mut bitboard: BitBoard;
    let white = board.color_combined(Color::White);
    let black = board.color_combined(Color::Black);
    let mut white_pieces = board.pieces(Knight) & white;
    let mut black_pieces = board.pieces(Knight) & black;
    let white_king_square = board.king_square(Color::White);
    let black_king_square = board.king_square(Color::Black);
    let mut total_material = 0;
    let blockers = white | black;

    let mut score = INVASION_BONUS
        * ((white & BLACK_HALF).popcnt() as i32 - (black & WHITE_HALF).popcnt() as i32);

    for square in white_pieces {
        bitboard = chess::get_knight_moves(square);
        total_material += 3;
        score += KNIGHT_TABLE[square.to_index()];
        score += evaluate_attack(bitboard, black_king_square);
    }
    for square in black_pieces {
        total_material += 3;
        bitboard = chess::get_knight_moves(square);
        score -= KNIGHT_TABLE[square.to_index()];
        score -= evaluate_attack(bitboard, white_king_square);
    }

    white_pieces = board.pieces(Bishop) & white;
    black_pieces = board.pieces(Bishop) & black;

    for square in white_pieces {
        total_material += 3;
        bitboard = chess::get_bishop_moves(square, blockers);
        score += BISHOP_VALUE + BISHOP_MOBILITY_VALUE * bitboard.popcnt() as i32;
        score += evaluate_attack(bitboard, black_king_square);
    }
    for square in black_pieces {
        total_material += 3;
        bitboard = chess::get_bishop_moves(square, blockers);
        score -= BISHOP_VALUE + BISHOP_MOBILITY_VALUE * bitboard.popcnt() as i32;
        score -= evaluate_attack(bitboard, white_king_square);
    }

    white_pieces = board.pieces(Queen) & white;
    black_pieces = board.pieces(Queen) & black;

    for square in white_pieces {
        total_material += 9;
        bitboard =
            chess::get_rook_moves(square, blockers) | chess::get_bishop_moves(square, blockers);
        score += QUEEN_VALUE + MOBILITY_VALUE * (bitboard.popcnt() as i32) / 2;
        score += evaluate_attack(bitboard, black_king_square);
    }
    for square in black_pieces {
        total_material += 9;
        bitboard =
            chess::get_rook_moves(square, blockers) | chess::get_bishop_moves(square, blockers);
        score -= QUEEN_VALUE + MOBILITY_VALUE * (bitboard.popcnt() as i32) / 2;
        score -= evaluate_attack(bitboard, white_king_square);
    }

    white_pieces = board.pieces(Rook) & white;
    black_pieces = board.pieces(Rook) & black;

    for square in white_pieces {
        total_material += 5;
        bitboard = chess::get_rook_moves(square, blockers);
        score += ROOK_TABLE_WHITE[square.to_index()] + MOBILITY_VALUE * bitboard.popcnt() as i32;
        score += evaluate_attack(bitboard, black_king_square);
    }
    for square in black_pieces {
        total_material += 5;
        bitboard = chess::get_rook_moves(square, blockers);
        score -= ROOK_TABLE_BLACK[square.to_index()] + MOBILITY_VALUE * bitboard.popcnt() as i32;
        score -= evaluate_attack(bitboard, white_king_square);
    }

    white_pieces = board.pieces(Pawn) & white;
    black_pieces = board.pieces(Pawn) & black;

    total_material += white_pieces.popcnt() as i32 + black_pieces.popcnt() as i32;

    let game_phase = (100 * total_material) / MAX_MATERIAL;

    score += game_phase * evaluate_pawn_shield(white_king_square, white_pieces) / 100;
    score -= game_phase * evaluate_pawn_shield(black_king_square, black_pieces) / 100;

    for square in white_pieces {
        score += evaluate_connected_pawns(square, Color::Black, &white_pieces);
        score += (PAWN_TABLE_OPENING_WHITE[square.to_index()] * game_phase
            + PAWN_TABLE_ENDGAME_WHITE[square.to_index()] * (100 - game_phase))
            / 100;
    }
    for square in black_pieces {
        score -= evaluate_connected_pawns(square, Color::Black, &black_pieces);
        score -= (PAWN_TABLE_OPENING_BLACK[square.to_index()] * game_phase
            + PAWN_TABLE_ENDGAME_BLACK[square.to_index()] * (100 - game_phase))
            / 100;
    }

    score += KING_TABLE_ENDGAME[white_king_square.to_index()] * (100 - game_phase) / 100;

    score += PIN_VALUE
        * ((board.pinned() & white).popcnt() as i32 - (board.pinned() & black).popcnt() as i32);

    if board.side_to_move() == Color::Black {
        score = -score
    };

    if *board.checkers() != EMPTY {
        score -= IN_CHECK_PENALTY;
    }

    score + SIDE_TO_MOVE_BONUS
}

#[inline]
fn evaluate_connected_pawns(square: Square, color: Color, pawns: &BitBoard) -> i32 {
    CONNECTED_PAWN_VALUE * chess::get_pawn_attacks(square, color, *pawns).popcnt() as i32
}

#[inline]
fn evaluate_pawn_shield(king_square: Square, pawns: BitBoard) -> i32 {
    PAWN_SHIELD_VALUE_TABLE[king_square.to_index()]
        * (chess::get_king_moves(king_square) & pawns).popcnt() as i32
}

#[inline]
fn evaluate_attack(attack_bitboard: BitBoard, king_square: Square) -> i32 {
    ATTACK_VALUE * (attack_bitboard & chess::get_king_moves(king_square)).popcnt() as i32
}
