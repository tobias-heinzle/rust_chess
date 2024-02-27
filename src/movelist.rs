use chess::{BitBoard, Board, ChessMove, MoveGen, Piece, EMPTY};

use crate::config::MVV_ORDERING;
use crate::search::{opponent_pieces_of_type, KillerMoves};

enum MoveOrderingStage {
    Hash,
    Captures,
    KillerOne,
    KillerTwo,
    Quiet,
}

pub struct MoveList<'a> {
    hash_move: Option<ChessMove>,
    killers: KillerMoves,
    stage: MoveOrderingStage,
    movegen: MoveGen,
    board: &'a Board,
    capture_index: usize,
}

impl MoveList<'_> {
    pub fn new(board: &Board, hash_move: Option<ChessMove>, killers: KillerMoves) -> MoveList {
        MoveList {
            hash_move,
            killers,
            stage: MoveOrderingStage::Hash,
            movegen: MoveGen::new_legal(board),
            board,
            capture_index: 0,
        }
    }
}

impl Iterator for MoveList<'_> {
    fn next(&mut self) -> Option<Self::Item> {
        match self.stage {
            MoveOrderingStage::Hash => {
                self.stage = MoveOrderingStage::Captures;
                self.movegen
                    .set_iterator_mask(get_targets(self.board, MVV_ORDERING[0]));

                match self.hash_move {
                    Some(hash_move) => match self.board.legal(hash_move) {
                        true => Some(hash_move),
                        false => self.next(),
                    },
                    None => self.next(),
                }
            }
            MoveOrderingStage::Captures => match self.movegen.next() {
                Some(chess_move) => match is_hash_move(self.hash_move, chess_move) {
                    true => self.next(),
                    false => Some(chess_move),
                },
                None => {
                    self.capture_index += 1;
                    if self.capture_index < MVV_ORDERING.len() {
                        self.movegen.set_iterator_mask(get_targets(
                            self.board,
                            MVV_ORDERING[self.capture_index],
                        ));
                    } else {
                        self.stage = MoveOrderingStage::KillerOne;
                    }
                    self.next()
                }
            },
            MoveOrderingStage::KillerOne => {
                self.stage = MoveOrderingStage::KillerTwo;
                match self.board.legal(self.killers.one) {
                    true => Some(self.killers.one),
                    false => self.next(),
                }
            }
            MoveOrderingStage::KillerTwo => {
                self.stage = MoveOrderingStage::Quiet;
                self.movegen.set_iterator_mask(!EMPTY);

                match self.board.legal(self.killers.two) {
                    true => Some(self.killers.two),
                    false => self.next(),
                }
                // match self.killer {
                //     Some(killer_move) => ,
                //     None => self.next(),
                // }
            }
            MoveOrderingStage::Quiet => match self.movegen.next() {
                Some(chess_move) => {
                    match is_hash_or_killer_move(self.killers, self.hash_move, chess_move) {
                        true => self.next(),
                        false => Some(chess_move),
                    }
                }
                None => None,
            },
        }
    }

    type Item = ChessMove;
}

#[inline]
fn is_hash_move(hash_move: Option<ChessMove>, chess_move: ChessMove) -> bool {
    match hash_move {
        Some(m) => m == chess_move,
        None => false,
    }
}

#[inline]
fn is_hash_or_killer_move(
    killers: KillerMoves,
    hash_move: Option<ChessMove>,
    chess_move: ChessMove,
) -> bool {
    chess_move == killers.one || chess_move == killers.two || is_hash_move(hash_move, chess_move)
}

#[inline]
fn get_targets(board: &Board, piece_type: Piece) -> BitBoard {
    match piece_type {
        Piece::Pawn => match board.en_passant() {
            Some(en_passant_square) => {
                opponent_pieces_of_type(piece_type, board)
                    | BitBoard::from_square(en_passant_square)
            }
            None => opponent_pieces_of_type(piece_type, board),
        },
        _ => opponent_pieces_of_type(piece_type, board),
    }
}

pub struct QuiescenceMoveList<'a> {
    movegen: MoveGen,
    board: &'a Board,
    capture_index: usize,
}

impl QuiescenceMoveList<'_> {
    pub fn new(board: &Board) -> QuiescenceMoveList {
        let mut movegen = MoveGen::new_legal(board);
        movegen.set_iterator_mask(get_targets(board, MVV_ORDERING[0]));
        QuiescenceMoveList {
            movegen,
            board,
            capture_index: 0,
        }
    }
}

impl Iterator for QuiescenceMoveList<'_> {
    fn next(&mut self) -> Option<Self::Item> {
        match self.movegen.next() {
            Some(chess_move) => Some(chess_move),
            None => {
                self.capture_index += 1;
                if self.capture_index < MVV_ORDERING.len() {
                    self.movegen.set_iterator_mask(get_targets(
                        self.board,
                        MVV_ORDERING[self.capture_index],
                    ));
                } else {
                    return None;
                }
                self.next()
            }
        }
    }

    type Item = ChessMove;
}
