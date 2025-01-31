use std::rc::Rc;
use std::vec;

use chess::{BitBoard, Board, ChessMove, MoveGen, Piece, EMPTY};

use crate::config::MVV_ORDERING;
use crate::search::{opponent_pieces_of_type, KillerMoves, Table64by64};

enum MoveOrderingStage {
    Hash,
    Captures,
    KillerOne,
    KillerTwo,
    Quiet,
}

pub struct MoveList<'a> {
    board: &'a Board,
    hash_move: Option<ChessMove>,
    killers: KillerMoves,
    history_table: Table64by64,
    quiets: Option<Vec<ChessMove>>,
    stage: MoveOrderingStage,
    movegen: MoveGen,
    capture_index: usize,
}

impl<'a> MoveList<'a> {
    pub fn new(
        board: &'a Board,
        hash_move: Option<ChessMove>,
        killers: KillerMoves,
        history_table: Table64by64,
    ) -> MoveList<'a> {
        MoveList {
            board,
            hash_move,
            killers,
            history_table,
            quiets: None,
            stage: MoveOrderingStage::Hash,
            movegen: MoveGen::new_legal(board),
            capture_index: 0,
        }
    }
}

impl<'a> Iterator for MoveList<'a> {
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
                Some(chess_move) => match self.hash_move == Some(chess_move) {
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
            }
            MoveOrderingStage::Quiet => match self.quiets.as_mut() {
                Some(quiet_moves) => match quiet_moves.pop() {
                    Some(chess_move) => {
                        match is_killer_or_hash(chess_move, self.killers, self.hash_move) {
                            true => self.next(),
                            false => Some(chess_move),
                        }
                    }

                    None => None,
                },
                None => {
                    let mut move_vector = vec![];
                    loop {
                        match self.movegen.next() {
                            Some(chess_move) => move_vector.push(chess_move), //self.quiets.as_mut()?.push(chess_move),
                            None => break,
                        }
                    }
                    move_vector.sort_unstable_by_key(|m| {
                        // TODO: sort by history heuristic
                        self.history_table[m.get_source().to_index()][m.get_dest().to_index()]
                        // + match m.get_promotion() {
                        //     None => 0,
                        //     Some(_) => 10,
                        // }
                    });
                    move_vector.reverse();
                    self.quiets = Some(move_vector);

                    self.next()
                }
            },
        }
    }

    type Item = ChessMove;
}

#[inline]
fn is_killer_or_hash(
    chess_move: ChessMove,
    killers: KillerMoves,
    hash_move: Option<ChessMove>,
) -> bool {
    chess_move == killers.one || chess_move == killers.two || hash_move == Some(chess_move)
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
