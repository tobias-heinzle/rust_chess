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
        let time_limit = 10;
        let (_, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let engine = rust_chess::ChessEngine{board: board, receiver_channel: rx, sender_channel: tx};
        
        let result = engine.root_search( max_depth, time_limit);

        assert_eq!(result.0, rust_chess::INFINITY);
        assert_eq!(result.1, chess::ChessMove::from_str("f6a6").expect("Invalid Move"));
        
    }

    #[test]
    fn mate_in_three_1(){
        let board = chess::Board::from_str("3r4/pR2N3/2pkb3/5p2/8/2B5/qP3PPP/4R1K1 w - - 1 1").expect("Invalid position");
        let max_depth = 6;
        let time_limit = 10;
        let (_x, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let engine = rust_chess::ChessEngine{board: board, receiver_channel: rx, sender_channel: tx};
        
        let result = engine.root_search( max_depth, time_limit);

        assert_eq!(result.0, rust_chess::INFINITY);
        assert_eq!(result.1, chess::ChessMove::from_str("c3e5").expect("Invalid Move"));
        
    }

    // #[test]
    // fn mate_in_four_0(){
    //     let board = chess::Board::from_str("r4r1k/1R1R2p1/7p/8/8/3Q1Ppq/P7/6K1 w - - 0 1").expect("Invalid position");
    //     let max_depth = 8;
    //     let time_limit = 10;
        
    //     let result = rust_chess::root_search(&board, max_depth, time_limit);

    //     assert_eq!(result.0, rust_chess::INFINITY);
    //     assert_eq!(result.1, chess::ChessMove::from_str("d3h7").expect("Invalid Move"));
        
    // }

    // #[test]
    // fn mate_in_five_0(){
    //     let board = chess::Board::from_str("2q1nk1r/4Rp2/1ppp1P2/6Pp/3p1B2/3P3P/PPP1Q3/6K1 w - - 0 1").expect("Invalid position");
    //     let max_depth = 10;
    //     let time_limit = 10;
        
    //     let result = rust_chess::root_search(&board, max_depth, time_limit);

    //     assert_eq!(result.0, rust_chess::INFINITY);
    //     assert_eq!(result.1, chess::ChessMove::from_str("e7e8").expect("Invalid Move"));
    // }

    // Mate in 12: 8/3P3k/n2K3p/2p3n1/1b4N1/2p1p1P1/8/3B4 w - - 0 1

}
