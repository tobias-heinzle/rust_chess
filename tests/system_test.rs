use std::str::FromStr;
use std::sync::{Arc, mpsc};
use rust_chess;
use chess;


fn setup_test_context(board: chess::Board) -> rust_chess::search::SearchContext {
    let (_, rx) = mpsc::channel();
    let (tx, _) = mpsc::channel();
    let hash_table = Arc::new(
        rust_chess::table::TranspositionTable::new(
            rust_chess::uci::HASH_TABLE_SIZE, 
            rust_chess::table::TableEntryData{
                best_move : chess::ChessMove::new(
                    chess::Square::A1, 
                    chess::Square::A1, 
                    None), 
                score : 0, 
                depth : 0, 
                score_bound : rust_chess::table::ScoreBound::LowerBound}
            )
        );
    
    return rust_chess::search::SearchContext::new(board, rx, tx, Arc::clone(&hash_table));
}

#[cfg(test)]
mod tests {
    

    use super::*;

    #[test]
    fn mate_in_three_0(){
        let board = chess::Board::from_str("r5rk/5p1p/5R2/4B3/8/8/7P/7K w - - 0 1").expect("Invalid position");
        let max_depth = 6;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);

        assert_eq!(result.1.to_string(), "f6a6");
        assert_eq!(result.0, rust_chess::search::INFINITY - 2);
        
    }

    #[test]
    fn mate_in_three_1(){
        let board = chess::Board::from_str("3r4/pR2N3/2pkb3/5p2/8/2B5/qP3PPP/4R1K1 w - - 1 1").expect("Invalid position");
        let max_depth = 6;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);
        assert_eq!(result.1.to_string(), "c3e5");
        assert_eq!(result.0, rust_chess::search::INFINITY - 2);
        
    }

    #[test]
    fn repetition_draw(){
        let board = chess::Board::from_str("kr4QQ/6QQ/6QQ/6QQ/6QQ/5PPP/4q3/7K b - - 0 1").expect("Invalid position");
        let max_depth = 6;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);
        assert_eq!(result.0, rust_chess::search::DRAW);
    }

    #[test]
    fn avoid_repetition() {
        let command: Vec<&str> = "fen R5k1/8/6pp/5p2/P4P2/r3P3/5KPP/8 b - - 1 43 moves g8g7 a8a7 g7g8".split(" ").collect();
        let position = rust_chess::uci::change_position(&command[0 ..]);
        let fen = position.board.to_string();
        let max_depth = 6;
        let mut context = setup_test_context(position.board);
        for hash in position.hash_history {
            context.set_visited(hash);
        }
        
        let result = context.root_search( max_depth);
        assert_eq!(fen, "6k1/R7/6pp/5p2/P4P2/r3P3/5KPP/8 w - - 0 1");
        assert!(result.1.to_string() != "a7a8");
    }

    #[test]
    fn is_repetition() {
        let command: Vec<&str> = "fen R5k1/8/6pp/5p2/P4P2/r3P3/5KPP/8 b - - 1 43 moves g8g7 a8a7 g7g8 a7a8".split(" ").collect();
        let position = rust_chess::uci::change_position(&command[0 ..]);
        let fen = position.board.to_string();
        let max_depth = 6;
        let mut context = setup_test_context(position.board);
        for hash in position.hash_history {
            context.set_visited(hash);
        }
        
        let result = context.root_search( max_depth);
        assert_eq!(fen, "R5k1/8/6pp/5p2/P4P2/r3P3/5KPP/8 b - - 0 1");
        assert!(result.0 == rust_chess::search::DRAW);
    }

    #[test]
    fn find_critical_endgame_move_part_1() {
        let board = chess::Board::from_str("8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1").expect("Invalid position");
        let max_depth = 18;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);
        assert_eq!(result.1.to_string(), "a1b1");
    
    }

    #[test]
    fn find_critical_endgame_move_part_2() {
        let board = chess::Board::from_str("8/1k6/3p4/p2P1p2/P2P1P2/8/8/1K6 w - - 2 2").expect("Invalid position");
        let max_depth = 18;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);
        assert_eq!(result.1.to_string(), "b1c1");
    
    }

    
    #[test]
    fn find_critical_endgame_move_part_3() {
        let board = chess::Board::from_str("8/2k5/3p4/p2P1p2/P2P1P2/8/8/2K5 w - - 0 1").expect("Invalid position");
        let max_depth = 18;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);
        assert_eq!(result.1.to_string(), "c1d1");
    
    }
    

    #[test]
    fn mate_in_four(){
        let board = chess::Board::from_str("r4r1k/1R1R2p1/7p/8/8/3Q1Ppq/P7/6K1 w - - 0 1").expect("Invalid position");
        let max_depth = 8;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);

        assert_eq!(result.1.to_string(), "d3h7");
        assert_eq!(result.0, rust_chess::search::INFINITY - 3);
        
    }


    #[test]
    fn mate_in_negative_one(){
        let board = chess::Board::from_str("6k1/5ppp/5n2/2p2P2/2P1p3/6q1/8/1B1r1K1R w - - 2 45").expect("Invalid position");
        let max_depth = 4;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);

        assert_eq!(result.1.to_string(), "f1e2");
        assert_eq!(result.0, -rust_chess::search::INFINITY + 1);
        
    }

    
    #[test]
    fn mate_in_negative_two(){
        let board = chess::Board::from_str("6k1/5ppp/5n2/2p2P2/2Prp3/8/2B1K1q1/7R w - - 2 45").expect("Invalid position");
        let max_depth = 4;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);

        assert_eq!(result.1.to_string(), "e2e1");
        assert_eq!(result.0, -rust_chess::search::INFINITY + 2);
        
    }

    #[test]
    fn blunder_vs_myopic_bot_1(){
        let board = chess::Board::from_str("4r1k1/p2nB2p/4pbp1/1Np5/8/1P3P2/P3KP1P/3R4 w - - 6 26").expect("Invalid position");
        let max_depth = 8;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);

        assert_eq!(result.1.to_string(), "d2d7");
        
    }


    #[test]
    fn blunder_vs_myopic_bot_2(){
        let board = chess::Board::from_str("8/p2N1nk1/8/4pKP1/1P5p/8/P7/8 b - - 1 47").expect("Invalid position");
        let max_depth = 10;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);

        assert_eq!(result.1.to_string(), "a2a4");
        
    }


    #[test]
    fn enemy_blunder_of_myopic_bot(){
        let board = chess::Board::from_str("8/p7/3n2k1/4K1P1/1P6/6N1/P6p/8 b - - 3 51").expect("Invalid position");
        let max_depth = 11;
        let mut context = setup_test_context(board);
        
        let result = context.root_search( max_depth);

        assert_eq!(result.1.to_string(), "g6g5");
        
    }

    
    // #[test]
    // fn mate_in_five(){
    //     let board = chess::Board::from_str("4nr1k/p1p1p1pp/bp1pn1r1/8/6QR/6RP/1BBq1PP1/6K1 w - - 0 1").expect("Invalid position");
    //     let max_depth = 10;
    //     let mut context = setup_test_context(board);
        
    //     let result = context.root_search( max_depth);

    //     assert_eq!(result.0, rust_chess::search::INFINITY  - 4);
    //     assert_eq!(result.1, chess::ChessMove::from_str("e2e1").expect("Invalid Move"));
        
    // }

    // #[test]
    // fn mate_in_five_1(){
    //     let board = chess::Board::from_str("2q1nk1r/4Rp2/1ppp1P2/6Pp/3p1B2/3P3P/PPP1Q3/6K1 w - - 0 1").expect("Invalid position");
    //     let max_depth = 9;
    //     let mut context = setup_test_context(board);
        
    //     let result = context.root_search( max_depth);

    //     assert_eq!(result.1, chess::ChessMove::from_str("e7e8").expect("Invalid Move"));
    //     assert_eq!(result.0, rust_chess::search::INFINITY - 5);
    // }

    // Mate in 12: 8/3P3k/n2K3p/2p3n1/1b4N1/2p1p1P1/8/3B4 w - - 0 1

}
