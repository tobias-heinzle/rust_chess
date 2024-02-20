use crate::search::{opponent_pieces, player_pieces, PositionScore};
use chess::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
use chess::{BitBoard, Board, Color, EMPTY};

const PAWN_VALUE: i32 = 80;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 310;
const ROOK_VALUE: i32 = 450;
const QUEEN_VALUE: i32 = 900;

const PIN_VALUE: i32 = 10;
const MOBILITY_VALUE: i32 = 1;
const IN_CHECK_PENALTY: i32 = 30;

const CENTER_PAWN_BONUS: i32 = 10;
const CENTER: BitBoard = BitBoard(0x1818000000);

const BAD_KING_PENALTY: i32 = 15;
const BAD_KING_ZONE: BitBoard = BitBoard(0x387cffffffff7c38);

const ADVANCED_PAWN_BONUS: i32 = 11;
const ADVANCED_PAWN_SQUARES_WHITE: BitBoard = BitBoard(0xffffff0000000000);
const ADVANCED_PAWN_SQUARES_BLACK: BitBoard = BitBoard(0xffffff);

const MAX_PIECES: i32 = 12;

const PAWN_TABLE: [i32; 64] = [
    -5, -5, -5, -5, -5, -5, -5, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5,
    10, 10, 5, 0, -5, -5, 0, 5, 10, 10, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -5, 0, 0, 0, 0, 0, 0,
    -5, -5, -5, -5, -5, -5, -5, -5, -5,
];
const PAWN_TABLE_OPENING_WHITE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 0, 0, 5, 5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 15, 0, 0,
    0, 0, 0, 0, 16, 16, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 20, 20, 20, 20, 20, 20, 20, 20, 0, 0, 0,
    0, 0, 0, 0, 0,
];
const PAWN_TABLE_OPENING_BLACK: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 20, 20, 20, 20, 20, 20, 20, 20, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 0, 16,
    16, 0, 0, 0, 0, 0, 0, 15, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 0, 0, 5, 5, 5, 0, 0, 0,
    0, 0, 0, 0, 0,
];
const PAWN_TABLE_ENDGAME_WHITE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -2, -2, -2, -2, 2, -2, -2, -2, 5, 5, 5, 5, 5, 5, 5, 5, 10, 10, 10, 10,
    10, 10, 10, 10, 15, 15, 15, 15, 15, 15, 15, 15, 25, 25, 25, 25, 25, 25, 25, 25, 50, 50, 50, 50,
    50, 50, 50, 50, 0, 0, 0, 0, 0, 0, 0, 0,
];
const PAWN_TABLE_ENDGAME_BLACK: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 25, 25, 25, 25, 25, 25, 25, 25, 15, 15,
    15, 15, 15, 15, 15, 15, 10, 10, 10, 10, 10, 10, 10, 10, 5, 5, 5, 5, 5, 5, 5, 5, -2, -2, -2, -2,
    2, -2, -2, -2, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_TABLE: [i32; 64] = [
    -5, -5, -5, -5, -5, -5, -5, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5,
    10, 10, 5, 0, -5, -5, 0, 5, 10, 10, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -5, 0, 0, 0, 0, 0, 0,
    -5, -5, -5, -5, -5, -5, -5, -5, -5,
];

const BISHOP_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 2, 2, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 3, 3, 5, 0, 0,
    0, 0, 5, 3, 3, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 2, 2, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const ROOK_TABLE_WHITE: [i32; 64] = [
    -1, -1, 5, 5, 5, 5, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 10, 10, 10, 10, 10, 10, 10, 0, 0, 0,
    0, 0, 0, 0, 0,
];

const ROOK_TABLE_BLACK: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 10, 10, 10, 10, 10, 10, 10, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, -1, 5, 5,
    5, 5, -1, -1,
];

const KING_TABLE_OPENING: [i32; 64] = [
    2, 2, 2, -5, -5, -5, 2, 2, 0, 0, 0, -4, -4, -4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -4, -4, -4, 0, 0, 5, 5, 5, -5,
    -5, -5, 5, 5,
];

const KING_TABLE_ENDGAME: [i32; 64] = [
    -5, -5, -5, -5, -5, -5, -5, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 2, 2, 2, 2, 0, -5, -5, 0, 2,
    3, 3, 2, 0, -5, -5, 0, 2, 3, 3, 2, 0, -5, -5, 0, 2, 2, 2, 2, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5,
    -5, -5, -5, -5, -5, -5, -5, -5,
];

#[inline]
pub fn new_evaluate(board: &Board) -> PositionScore {
    // TODO: PSQT for king (use get king or something like that) and pawns(check how many pawns in an area with &) interpolate between endgame and earlygame
    //       Draw by insufficient material (no pawns and total material <= bishop)
    //       Endgame bring in king by scoring distance (len distance between kings is negative, center good, edege bad (via bitmask)) early game corners good via bitmask (skip castling square before castling)

    // let mut score = 0;

    let all_player = player_pieces(board);
    let all_opponent = opponent_pieces(board);
    let blockers = all_player | all_opponent;

    // Pawn evaluation
    let mut player = board.pieces(Pawn) & all_player;
    let mut opponent = board.pieces(Pawn) & all_opponent;

    let mut total_pieces = 0;

    let mut score = match board.side_to_move() {
        Color::White => {
            ADVANCED_PAWN_BONUS
                * (advanced_pawn_count(&player, Color::White)
                    - advanced_pawn_count(&opponent, Color::Black))
        }
        Color::Black => {
            ADVANCED_PAWN_BONUS
                * (advanced_pawn_count(&player, Color::Black)
                    - advanced_pawn_count(&opponent, Color::White))
        }
    };
    score += CENTER_PAWN_BONUS * (player & CENTER).popcnt() as i32;
    score -= CENTER_PAWN_BONUS * (opponent & CENTER).popcnt() as i32;
    score += PAWN_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    // Knight evalutation
    for square in board.pieces(Knight) & all_player {
        total_pieces += 1;
        score += KNIGHT_VALUE
            + KNIGHT_TABLE[square.to_index()]
            + MOBILITY_VALUE * chess::get_knight_moves(square).popcnt() as i32;
    }
    for square in board.pieces(Knight) & all_opponent {
        total_pieces += 1;
        score -= KNIGHT_VALUE
            + KNIGHT_TABLE[square.to_index()]
            + MOBILITY_VALUE * chess::get_knight_moves(square).popcnt() as i32;
    }

    // Bishop evalutation
    for square in board.pieces(Bishop) & all_player {
        total_pieces += 1;
        score += BISHOP_VALUE
            + BISHOP_TABLE[square.to_index()]
            + MOBILITY_VALUE * chess::get_bishop_moves(square, blockers).popcnt() as i32;
    }
    for square in board.pieces(Bishop) & all_opponent {
        total_pieces += 1;
        score -= BISHOP_VALUE
            + BISHOP_TABLE[square.to_index()]
            + MOBILITY_VALUE * chess::get_bishop_moves(square, blockers).popcnt() as i32;
    }

    // Rook evalutation

    for square in board.pieces(Rook) & all_player {
        total_pieces += 1;
        score +=
            ROOK_VALUE + MOBILITY_VALUE * chess::get_rook_moves(square, blockers).popcnt() as i32;
    }
    for square in board.pieces(Rook) & all_opponent {
        total_pieces += 1;
        score -=
            ROOK_VALUE + MOBILITY_VALUE * chess::get_rook_moves(square, blockers).popcnt() as i32;
    }

    // Queen evaluation
    player = board.pieces(Queen) & all_player;
    opponent = board.pieces(Queen) & all_opponent;

    score += QUEEN_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    score += PIN_VALUE * (board.pinned() & all_opponent).popcnt() as i32;
    score -= PIN_VALUE * (board.pinned() & all_player).popcnt() as i32;

    // King evaluation

    let player_king_square = board.king_square(board.side_to_move()).to_index();
    score += (KING_TABLE_OPENING[player_king_square] * (100 * total_pieces) / MAX_PIECES
        + KING_TABLE_ENDGAME[player_king_square] * (100 - 100 * total_pieces) / MAX_PIECES)
        / 100;
    let opponent_king_square = board.king_square(!board.side_to_move()).to_index();
    score += (KING_TABLE_OPENING[opponent_king_square] * (100 * total_pieces) / MAX_PIECES
        + KING_TABLE_ENDGAME[opponent_king_square] * (100 - 100 * total_pieces) / MAX_PIECES)
        / 100;

    // score -= BAD_KING_PENALTY * (player & BAD_KING_ZONE).popcnt() as i32;
    // score += BAD_KING_PENALTY * (opponent & BAD_KING_ZONE).popcnt() as i32;

    if *board.checkers() != EMPTY {
        score -= IN_CHECK_PENALTY;
    }

    score
}

#[inline]
fn advanced_pawn_count(pieces: &BitBoard, color: Color) -> i32 {
    match color {
        Color::White => pieces & ADVANCED_PAWN_SQUARES_WHITE,
        Color::Black => pieces & ADVANCED_PAWN_SQUARES_BLACK,
    }
    .popcnt() as i32
}

///////////////////////////#################################################

#[inline]
pub fn evaluate(board: &Board) -> PositionScore {
    // TODO: PSQT for king (use get king or something like that) and pawns(check how many pawns in an area with &) interpolate between endgame and earlygame
    //       Draw by insufficient material (no pawns and total material <= bishop)
    //       Endgame bring in king by scoring distance (len distance between kings is negative, center good, edege bad (via bitmask)) early game corners good via bitmask (skip castling square before castling)

    // let mut score = 0;
    let white = board.color_combined(Color::White);
    let black = board.color_combined(Color::Black);
    let mut white_pieces = board.pieces(Knight) & white;
    let mut black_pieces = board.pieces(Knight) & black;
    let mut total_pieces = 0;
    let blockers = white | black;

    let mut score = 0;

    for square in white_pieces {
        total_pieces += 1;
        score += KNIGHT_VALUE
            + KNIGHT_TABLE[square.to_index()]
            + MOBILITY_VALUE * chess::get_knight_moves(square).popcnt() as i32;
    }
    for square in black_pieces {
        total_pieces += 1;
        score -= KNIGHT_VALUE
            + KNIGHT_TABLE[square.to_index()]
            + MOBILITY_VALUE * chess::get_knight_moves(square).popcnt() as i32;
    }

    white_pieces = board.pieces(Bishop) & white;
    black_pieces = board.pieces(Bishop) & black;

    // score += BISHOP_VALUE * (white_pieces.popcnt() as i32 - black_pieces.popcnt() as i32);

    for square in white_pieces {
        total_pieces += 1;
        score += BISHOP_VALUE
            + BISHOP_TABLE[square.to_index()]
            + MOBILITY_VALUE * chess::get_bishop_moves(square, blockers).popcnt() as i32;
    }
    for square in black_pieces {
        total_pieces += 1;
        score -= BISHOP_VALUE
            + BISHOP_TABLE[square.to_index()]
            + MOBILITY_VALUE * chess::get_bishop_moves(square, blockers).popcnt() as i32;
    }

    white_pieces = board.pieces(Rook) & white;
    black_pieces = board.pieces(Rook) & black;

    // score += ROOK_VALUE * (white_pieces.popcnt() as i32 - black_pieces.popcnt() as i32);

    for square in white_pieces {
        total_pieces += 1;
        score += ROOK_VALUE
            + ROOK_TABLE_WHITE[square.to_index()]
            + MOBILITY_VALUE * chess::get_rook_moves(square, blockers).popcnt() as i32;
    }
    for square in black_pieces {
        total_pieces += 1;
        score -= ROOK_VALUE
            + ROOK_TABLE_BLACK[square.to_index()]
            + MOBILITY_VALUE * chess::get_rook_moves(square, blockers).popcnt() as i32;
    }

    white_pieces = board.pieces(Queen) & white;
    black_pieces = board.pieces(Queen) & black;

    score += QUEEN_VALUE * (white_pieces.popcnt() as i32 - black_pieces.popcnt() as i32);

    score += PIN_VALUE
        * ((board.pinned() & white_pieces).popcnt() as i32
            - (board.pinned() & black_pieces).popcnt() as i32);

    white_pieces = board.pieces(Pawn) & white;
    black_pieces = board.pieces(Pawn) & black;

    let game_phase = (100 * total_pieces) / MAX_PIECES;

    for square in white_pieces {
        score += PAWN_VALUE;
        score += (PAWN_TABLE_OPENING_WHITE[square.to_index()] * game_phase
            + PAWN_TABLE_ENDGAME_WHITE[square.to_index()] * (100 - game_phase))
            / 100;
    }
    for square in black_pieces {
        score -= PAWN_VALUE;
        score -= (PAWN_TABLE_OPENING_BLACK[square.to_index()] * game_phase
            + PAWN_TABLE_ENDGAME_BLACK[square.to_index()] * (100 - game_phase))
            / 100;
    }

    let white_king_square = board.king_square(board.side_to_move()).to_index();
    score += (KING_TABLE_OPENING[white_king_square] * game_phase
        + KING_TABLE_ENDGAME[white_king_square] * (100 - game_phase))
        / 100;
    let black_king_square = board.king_square(!board.side_to_move()).to_index();
    score += (KING_TABLE_OPENING[black_king_square] * game_phase
        + KING_TABLE_ENDGAME[black_king_square] * (100 - game_phase))
        / 100;

    if board.side_to_move() == Color::Black {
        score = -score
    };

    if *board.checkers() != EMPTY {
        score -= IN_CHECK_PENALTY;
    }

    score
}
