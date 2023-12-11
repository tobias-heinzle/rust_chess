use main::root_search;

#[cfg(test)]
mod tests {
    #[test]
    fn mate_in_three(){
        let board = from_str("r5rk/5p1p/5R2/4B3/8/8/7P/7K w - - 0 1").expect("Invalid Position");
        let max_depth = 6;
        
        result = root_search(&board, max_depth);

        assert_eq!(result.1, chess::ChessMove.from_str("f6a6"));
        assert_eq!(result.0, 100000);
    }
}
