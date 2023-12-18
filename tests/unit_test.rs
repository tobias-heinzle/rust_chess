use std::str::FromStr;
use std::sync::mpsc;
use rust_chess;
use chess;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiescence_checkmate(){
        let board = chess::Board::from_str("R6k/6rp/5B2/8/8/8/7P/7K b - - 0 3").expect("Invalid position");
        let alpha = 0;
        let beta = 100;
        let (_, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let engine = rust_chess::ChessEngine{board: board, receiver_channel: rx, sender_channel: tx};
        
        let result = engine.quiescence_search(&board, alpha, beta);

        assert_eq!(result, -rust_chess::INFINITY);
    }

   
    #[test]
    fn quiescence_stalemate(){
        let board = chess::Board::from_str("7k/6Rp/7B/8/8/8/7P/7K b - - 0 1").expect("Invalid position");
        let alpha = 0;
        let beta = 100;

        let (_, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let engine = rust_chess::ChessEngine{board: board, receiver_channel: rx, sender_channel: tx};
        
        let result = engine.quiescence_search(&board, alpha, beta);

        assert_eq!(result, 0);
    }

}
