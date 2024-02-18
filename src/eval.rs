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

#[inline]
pub fn evaluate(board: &Board) -> PositionScore {
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
    player = board.pieces(Knight) & all_player;
    opponent = board.pieces(Knight) & all_opponent;

    score += KNIGHT_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    for square in player {
        score += MOBILITY_VALUE * chess::get_knight_moves(square).popcnt() as i32;
    }
    for square in opponent {
        score -= MOBILITY_VALUE * chess::get_knight_moves(square).popcnt() as i32;
    }

    // Bishop evalutation
    player = board.pieces(Bishop) & all_player;
    opponent = board.pieces(Bishop) & all_opponent;

    score += BISHOP_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    for square in player {
        score += MOBILITY_VALUE * chess::get_bishop_moves(square, blockers).popcnt() as i32;
    }
    for square in opponent {
        score -= MOBILITY_VALUE * chess::get_bishop_moves(square, blockers).popcnt() as i32;
    }

    // Rook evalutation
    player = board.pieces(Rook) & all_player;
    opponent = board.pieces(Rook) & all_opponent;

    score += ROOK_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    for square in player {
        score += MOBILITY_VALUE * chess::get_rook_moves(square, blockers).popcnt() as i32;
    }
    for square in opponent {
        score -= MOBILITY_VALUE * chess::get_rook_moves(square, blockers).popcnt() as i32;
    }

    // Queen evaluation
    player = board.pieces(Queen) & all_player;
    opponent = board.pieces(Queen) & all_opponent;

    score += QUEEN_VALUE * (player.popcnt() as i32 - opponent.popcnt() as i32);

    score += PIN_VALUE * (board.pinned() & all_opponent).popcnt() as i32;
    score -= PIN_VALUE * (board.pinned() & all_player).popcnt() as i32;

    // King evaluation
    player = board.pieces(King) & all_player;
    opponent = board.pieces(King) & all_opponent;

    score -= BAD_KING_PENALTY * (player & BAD_KING_ZONE).popcnt() as i32;
    score += BAD_KING_PENALTY * (opponent & BAD_KING_ZONE).popcnt() as i32;

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
