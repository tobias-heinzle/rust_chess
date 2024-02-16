use crate::search::{opponent_pieces, player_pieces, PositionScore};
use chess::{Board, Piece, EMPTY};

const PAWN_VALUE: i32 = 80;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 305;
const ROOK_VALUE: i32 = 450;
const QUEEN_VALUE: i32 = 900;

const PIN_VALUE: i32 = 10;
const MOBILITY_VALUE: i32 = 1;
const IN_CHECK_PENALTY: i32 = 30;

#[inline]
pub fn evaluate(board: &Board) -> PositionScore {
    // TODO: PSQT for king (use get king or something like that) and pawns(check how many pawns in an area with &) interpolate between endgame and earlygame
    //       Draw by insufficient material (no pawns and total material <= bishop)
    //       Endgame bring in king by scoring distance (len distance between kings is negative, center good, edege bad (via bitmask)) early game corners good via bitmask (skip castling square before castling)

    let mut score = 0;

    let all_player = player_pieces(board);
    let all_opponent = opponent_pieces(board);
    let blockers = all_player | all_opponent;

    // Pawn evaluation
    let mut player = board.pieces(Piece::Pawn) & all_player;
    let mut opponent = board.pieces(Piece::Pawn) & all_opponent;

    score += PAWN_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    // Knight evalutation
    player = board.pieces(Piece::Knight) & all_player;
    opponent = board.pieces(Piece::Knight) & all_opponent;

    score += KNIGHT_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    for square in player {
        score += MOBILITY_VALUE * chess::get_knight_moves(square).popcnt() as i32;
    }
    for square in opponent {
        score -= MOBILITY_VALUE * chess::get_knight_moves(square).popcnt() as i32;
    }

    // Bishop evalutation
    player = board.pieces(Piece::Bishop) & all_player;
    opponent = board.pieces(Piece::Bishop) & all_opponent;

    score += BISHOP_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    for square in player {
        score += MOBILITY_VALUE * chess::get_bishop_moves(square, blockers).popcnt() as i32;
    }
    for square in opponent {
        score -= MOBILITY_VALUE * chess::get_bishop_moves(square, blockers).popcnt() as i32;
    }

    // Rook evalutation
    player = board.pieces(Piece::Rook) & all_player;
    opponent = board.pieces(Piece::Rook) & all_opponent;

    score += ROOK_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    for square in player {
        score += MOBILITY_VALUE * chess::get_rook_moves(square, blockers).popcnt() as i32;
    }
    for square in opponent {
        score -= MOBILITY_VALUE * chess::get_rook_moves(square, blockers).popcnt() as i32;
    }

    // Queen evaluation
    player = board.pieces(Piece::Queen) & all_player;
    opponent = board.pieces(Piece::Queen) & all_opponent;

    score += QUEEN_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    score += PIN_VALUE * (board.pinned() & all_opponent).popcnt() as i32;
    score -= PIN_VALUE * (board.pinned() & all_player).popcnt() as i32;

    if *board.checkers() != EMPTY {
        score -= IN_CHECK_PENALTY;
    }

    score
}
