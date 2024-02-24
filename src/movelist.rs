use chess::{BitBoard, Board, ChessMove, MoveGen, Piece, EMPTY};

use crate::{config::MVV_ORDERING, search::opponent_pieces_of_type};

enum MoveOrderingStage {
    Hash,
    Captures,
    Killer,
    Quiet,
}

pub struct MoveList<'a> {
    //capture_order: Vec<Piece>,
    hash_move: Option<ChessMove>,
    killer: Option<ChessMove>,
    stage: MoveOrderingStage,
    movegen: MoveGen,
    board: &'a Board,
    capture_index: usize,
}

impl MoveList<'_> {
    pub fn new(board: &Board, hash_move: Option<ChessMove>, killer: Option<ChessMove>) -> MoveList {
        MoveList {
            //capture_order,
            hash_move,
            killer,
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
                Some(chess_move) => Some(chess_move),
                None => {
                    self.capture_index += 1;
                    if self.capture_index < MVV_ORDERING.len() {
                        self.movegen.set_iterator_mask(get_targets(
                            self.board,
                            MVV_ORDERING[self.capture_index],
                        ));
                    } else {
                        self.stage = MoveOrderingStage::Killer;
                    }
                    self.next()
                }
            },
            MoveOrderingStage::Killer => {
                self.stage = MoveOrderingStage::Quiet;
                self.movegen.set_iterator_mask(!EMPTY);

                match self.killer {
                    Some(killer_move) => match self.board.legal(killer_move) {
                        true => self.killer,
                        false => self.next(),
                    },
                    None => self.next(),
                }
            }
            MoveOrderingStage::Quiet => self.movegen.next(),
        }
    }

    type Item = ChessMove;
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
