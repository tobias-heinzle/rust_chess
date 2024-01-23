use std::str::FromStr;
use rust_chess;
use chess;

use rust_chess::test_utils::setup_test_context;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiescence_checkmate(){
        let board = chess::Board::from_str("R6k/6rp/5B2/8/8/8/7P/7K b - - 0 3").expect("Invalid position");
        let alpha = 0;
        let beta = 100;
        let mut context = setup_test_context(board);

        let result = context.quiescence_search(&board, alpha, beta);

        assert_eq!(result, -(rust_chess::search::INFINITY));
    }

   
    #[test]
    fn quiescence_stalemate(){
        let board = chess::Board::from_str("7k/6Rp/7B/8/8/8/7P/7K b - - 0 1").expect("Invalid position");
        let alpha = 0;
        let beta = 100;
        let mut context = setup_test_context(board);

        let result = context.quiescence_search(&board, alpha, beta);

        assert_eq!(result, 0);
    }

    #[test]
    fn uci_read_position(){
        let command: Vec<&str> = "fen 7k/6Rp/7B/8/8/8/7P/7K w - - 0 1 moves g7h7 h8h7 h6g7".split(" ").collect();
        
        let position = rust_chess::uci::change_position(&command[0 ..]);
        
        // Board struct does not track the move number, so there is no halfmove of fullmove clock in the struct*
        let new_board = position.board;
        let resulting_position = "8/6Bk/8/8/8/8/7P/7K b - - 0 1".to_string();
        assert_eq!(format!("{new_board}"), resulting_position)

    }

    #[test]
    fn test_extensions_in_check(){
        let board = chess::Board::from_str("r7/ppp3k1/6b1/3pQ3/5R2/1P4P1/P5PP/6K1 b - - 4 43").expect("Invalid position");

        let extension = rust_chess::search::extend_check(&board, 0);

        assert_eq!(extension, true);
    }

    #[test]
    fn test_extensions_not_in_check(){
        let board = chess::Board::from_str("r5k1/ppp5/5Qb1/3p4/5R2/1P4P1/P5PP/6K1 b - - 6 44").expect("Invalid position");

        let extension = rust_chess::search::extend_check(&board, 0);

        assert_eq!(extension, false);
    }

    #[test]
    fn test_extensions_one_to_many(){
        let board = chess::Board::from_str("r5k1/ppp5/5Qb1/3p4/5R2/1P4P1/P5PP/6K1 b - - 6 44").expect("Invalid position");

        let extension = rust_chess::search::extend_check(&board, 3);

        assert_eq!(extension, false);
    }

}
