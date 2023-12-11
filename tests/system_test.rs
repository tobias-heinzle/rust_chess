use std::str::FromStr;
use rust_chess;
use chess;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mate_in_three_0(){
        let board = chess::Board::from_str("r5rk/5p1p/5R2/4B3/8/8/7P/7K w - - 0 1").expect("Invalid position");
        let max_depth = 6;
        
        let result = rust_chess::root_search(&board, max_depth);

        assert_eq!(result.0, 100000);
        assert_eq!(result.1, chess::ChessMove::from_str("f6a6").expect("Invalid Move"));
        
    }

    #[test]
    fn mate_in_three_1(){
        let board = chess::Board::from_str("3r4/pR2N3/2pkb3/5p2/8/2B5/qP3PPP/4R1K1 w - - 1 1").expect("Invalid position");
        let max_depth = 6;
        
        let result = rust_chess::root_search(&board, max_depth);

        assert_eq!(result.0, 100000);
        assert_eq!(result.1, chess::ChessMove::from_str("c3e5").expect("Invalid Move"));
        
    }

    #[test]
    fn mate_in_four_0(){
        let board = chess::Board::from_str("r4r1k/1R1R2p1/7p/8/8/3Q1Ppq/P7/6K1 w - - 0 1").expect("Invalid position");
        let max_depth = 8;
        
        let result = rust_chess::root_search(&board, max_depth);

        assert_eq!(result.0, 100000);
        assert_eq!(result.1, chess::ChessMove::from_str("c3e5").expect("Invalid Move"));
        
    }

}
