use std::str::FromStr;
use std::sync::mpsc;
use rust_chess;
use chess;

#[cfg(test)]
mod tests {
    

    use super::*;

    #[test]
    fn mate_in_three_0(){
        let board = chess::Board::from_str("r5rk/5p1p/5R2/4B3/8/8/7P/7K w - - 0 1").expect("Invalid position");
        let max_depth = 6;
        let (_, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let mut context = rust_chess::search::SearchContext::new(board, rx, tx);
        
        let result = context.root_search( max_depth);

        assert_eq!(result.1, chess::ChessMove::from_str("f6a6").expect("Invalid Move"));
        assert_eq!(result.0, rust_chess::search::INFINITY - 3);
        
    }

    #[test]
    fn mate_in_three_1(){
        let board = chess::Board::from_str("3r4/pR2N3/2pkb3/5p2/8/2B5/qP3PPP/4R1K1 w - - 1 1").expect("Invalid position");
        let max_depth = 6;
        let (_x, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let mut context = rust_chess::search::SearchContext::new(board, rx, tx);
        
        let result = context.root_search( max_depth);
        assert_eq!(result.1, chess::ChessMove::from_str("c3e5").expect("Invalid Move"));
        assert_eq!(result.0, rust_chess::search::INFINITY - 3);
        
    }

    #[test]
    fn repetition_draw(){
        let board = chess::Board::from_str("kr4QQ/6QQ/6QQ/6QQ/6QQ/5PPP/4q3/7K b - - 0 1").expect("Invalid position");
        let max_depth = 6;
        let (_x, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let mut context = rust_chess::search::SearchContext::new(board, rx, tx);
        
        let result = context.root_search( max_depth);
        assert_eq!(result.0, rust_chess::search::DRAW);
    }

    #[test]
    fn avoid_repetition() {
        let command: Vec<&str> = "fen R5k1/8/6pp/5p2/P4P2/r3P3/5KPP/8 b - - 1 43 moves g8g7 a8a7 g7g8".split(" ").collect();
        let (board, position_hashes) = rust_chess::uci::change_position(&command[0 ..]);
        let fen = board.to_string();
        let max_depth = 6;
        let (_x, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let mut context = rust_chess::search::SearchContext::new(board, rx, tx);
        for hash in position_hashes.iter() {
            context.set_visited(*hash);
        }
        
        let result = context.root_search( max_depth);
        assert_eq!(fen, "6k1/R7/6pp/5p2/P4P2/r3P3/5KPP/8 w - - 0 1");
        assert!(result.1 != chess::ChessMove::from_str("a7a8").expect("Invalid Move"));
    }

    #[test]
    fn is_repetition() {
        let command: Vec<&str> = "fen R5k1/8/6pp/5p2/P4P2/r3P3/5KPP/8 b - - 1 43 moves g8g7 a8a7 g7g8 a7a8".split(" ").collect();
        let (board, position_hashes) = rust_chess::uci::change_position(&command[0 ..]);
        let fen = board.to_string();
        let max_depth = 6;
        let (_x, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let mut context = rust_chess::search::SearchContext::new(board, rx, tx);
        for hash in position_hashes.iter() {
            context.set_visited(*hash);
        }
        
        let result = context.root_search( max_depth);
        assert_eq!(fen, "R5k1/8/6pp/5p2/P4P2/r3P3/5KPP/8 b - - 0 1");
        assert!(result.0 == rust_chess::search::DRAW);
    }

    #[test]
    fn find_critical_endgame_move_part_1() {
        let board = chess::Board::from_str("8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1").expect("Invalid position");
        let max_depth = 18;
        let (_x, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let mut context = rust_chess::search::SearchContext::new(board, rx, tx);
        
        let result = context.root_search( max_depth);
        assert_eq!(result.1, chess::ChessMove::from_str("a1b1").expect("Invalid Move"));
    
    }

    #[test]
    fn find_critical_endgame_move_part_2() {
        let board = chess::Board::from_str("8/1k6/3p4/p2P1p2/P2P1P2/8/8/1K6 w - - 2 2").expect("Invalid position");
        let max_depth = 18;
        let (_x, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let mut context = rust_chess::search::SearchContext::new(board, rx, tx);
        
        let result = context.root_search( max_depth);
        assert_eq!(result.1, chess::ChessMove::from_str("b1c1").expect("Invalid Move"));
    
    }
    

    #[test]
    fn mate_in_four_0(){
        let board = chess::Board::from_str("r4r1k/1R1R2p1/7p/8/8/3Q1Ppq/P7/6K1 w - - 0 1").expect("Invalid position");
        let max_depth = 8;
        let (_x, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let mut context = rust_chess::search::SearchContext::new(board, rx, tx);
        
        let result = context.root_search( max_depth);

        assert_eq!(result.1, chess::ChessMove::from_str("d3h7").expect("Invalid Move"));
        assert_eq!(result.0, rust_chess::search::INFINITY - 4);
        
    }

    // #[test]
    // fn mate_in_five_0(){
    //     let board = chess::Board::from_str("2q1nk1r/4Rp2/1ppp1P2/6Pp/3p1B2/3P3P/PPP1Q3/6K1 w - - 0 1").expect("Invalid position");
    //     let max_depth = 9;
    //     let (_x, rx) = mpsc::channel();
    //     let (tx, _) = mpsc::channel();
    //     let mut context = rust_chess::search::SearchContext::new(board, rx, tx);
        
    //     let result = context.root_search( max_depth);

    //     assert_eq!(result.1, chess::ChessMove::from_str("e7e8").expect("Invalid Move"));
    //     assert_eq!(result.0, rust_chess::search::INFINITY - 5);
    // }

    // Mate in 12: 8/3P3k/n2K3p/2p3n1/1b4N1/2p1p1P1/8/3B4 w - - 0 1

}
