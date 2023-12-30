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
        let context = rust_chess::search::SearchContext{board: board, receiver_channel: rx, sender_channel: tx};
        
        let result = context.quiescence_search(&board, alpha, beta);

        assert_eq!(result, -rust_chess::search::INFINITY);
    }

   
    #[test]
    fn quiescence_stalemate(){
        let board = chess::Board::from_str("7k/6Rp/7B/8/8/8/7P/7K b - - 0 1").expect("Invalid position");
        let alpha = 0;
        let beta = 100;

        let (_, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let context = rust_chess::search::SearchContext{board: board, receiver_channel: rx, sender_channel: tx};
        
        let result = context.quiescence_search(&board, alpha, beta);

        assert_eq!(result, 0);
    }

    #[test]
    fn uci_read_position(){
        let command: Vec<&str> = "fen 7k/6Rp/7B/8/8/8/7P/7K w - - 0 1 moves g7h7 h8h7 h6g7".split(" ").collect();
        
        let new_board = rust_chess::uci::change_position(&command[0 ..]);
        
        // Board struct does not track the move number, so there is no halfmove of fullmove clock in the struct*
        let resulting_position = "8/6Bk/8/8/8/8/7P/7K b - - 0 1".to_string();
        assert_eq!(format!("{new_board}"), resulting_position)

    }

}
