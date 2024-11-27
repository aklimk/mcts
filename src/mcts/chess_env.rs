use crate::game_state_trait::{GameResult, GameState};
use chess::{Board, ChessMove, Color, MoveGen, Piece};
use std::str::FromStr;


/// Holds the state of the chessboard.
///
/// Wraps the chess module's `Board` with the addition of:
/// Tracking 50 move rule.
/// Tracking the last move.
#[derive(Debug, Clone)]
pub struct ChessState {
    pub board: Board,
    pub fifty_move_counter: u16,
    pub last_move: Option<ChessMove>
}


/// Allows the MCTS engine to build ChessState trees.
impl GameState<ChessMove> for ChessState {
    fn from_str(starting_fen: String) -> Self {
        return ChessState {
            board: Board::from_str(&starting_fen).unwrap(), 
            fifty_move_counter: 0, 
            last_move: None, 
        };
    }
    
    fn apply_action(&self, action: &ChessMove) -> Self {
        let mut new_fifty_move_counter = self.fifty_move_counter + 1;

        // Any capture or pawn move should reset the 50 move counter.        
        let is_capture = self.board.piece_on(action.get_dest()).is_some();
        
        let src_piece = self.board.piece_on(action.get_source());
        let pawn_moved = src_piece.is_some() && src_piece.unwrap() == Piece::Pawn;

        if is_capture || pawn_moved {
            new_fifty_move_counter = 0;
        }
       
        return ChessState {
            board: self.board.make_move_new(*action), 
            fifty_move_counter: new_fifty_move_counter, 
            last_move: Some(*action), 
        };
    }

    fn status_with_moves_left(&self) -> bool {
        // If there are still legal moves left, the game can still end up as a draw
        // due to the 50 move rule. The 3 fold repition rule is not considered.
        if self.fifty_move_counter >= 50 {
            return false;
        }
        return true;
    }

    fn result(&self) -> GameResult {
        // There are no legal moves left, so if there are no checks the game is a draw.
        if self.board.checkers() == &chess::EMPTY || self.fifty_move_counter >= 50 {
            return GameResult::Draw;
        } 
        if self.board.side_to_move() == Color::Black {
            return GameResult::FirstPlayerWin;
        }
        else {
            return GameResult::SecondPlayerWin;
        }
    }

    fn generate_legal_actions(&self) -> Vec<ChessMove> {
        return MoveGen::new_legal(&self.board).collect();
    }

    fn side_to_move(&self) -> bool {
        return self.board.side_to_move() == Color::White;
    }
}



/// Defines unit tests for the GameState implementation of
/// ChessState.
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use chess::Piece;
    use chess::Square;
    use chess::Rank;
    use chess::File;

    /// Test if the starting position generates correctly.
    #[test]
    fn test_generate_root() {
        let ground_truth = Board::from_str(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        ).unwrap();
        let test = ChessState::from_str(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
        ).board;
        assert!(test == ground_truth);
    }
    
    /// A series of tests to check if the logic behind making moves is correct.

    /// Tests e4 and nf3 opening moves.
    #[test]
    fn test_apply_action_new_from_start() {
        let starting_state = ChessState::from_str(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
        );
        
        let ground_truth = Board::from_str(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
        ).unwrap();
        let test = starting_state.apply_action(&ChessMove::new(
            Square::make_square(Rank::Second, File::E), 
            Square::make_square(Rank::Fourth, File::E),
            None
        ));
        assert!(test.board == ground_truth);

        let ground_truth = Board::from_str(
            "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1"
        ).unwrap();
        let test = starting_state.apply_action(&ChessMove::new(
            Square::make_square(Rank::First, File::G), 
            Square::make_square(Rank::Third, File::F),
            None
        ));
        assert!(test.board == ground_truth);
    }

    /// Tests to see if ending positions are correctly produced.
    #[test]
    fn test_apply_action_new_before_end() {
        let before_fools_mate = ChessState::from_str(
            "rnbqkbnr/pppp1ppp/4p3/8/5P2/4P3/PPPP2PP/RNBQKBNR b KQkq - 0 2".to_string()
        );

        let ground_truth = Board::from_str(
            "rnb1kbnr/pppp1ppp/4p3/8/5P1q/4P3/PPPP2PP/RNBQKBNR w KQkq - 1 3"
        ).unwrap();
        let test = before_fools_mate.apply_action(&ChessMove::new(
            Square::make_square(Rank::Eighth, File::D), 
            Square::make_square(Rank::Fourth, File::H),
            None
        ));
        assert!(test.board == ground_truth);
    }

    /// Tests to see if enpassant is handled correctly.
    #[test]
    fn test_apply_action_new_enpassant() {
        let before_en_passant = ChessState::from_str(
            "rnbqkbnr/pppp2pp/4p3/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3".to_string()
        );

        let ground_truth = Board::from_str(
            "rnbqkbnr/pppp2pp/4pP2/8/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 3"
        ).unwrap();
        let test = before_en_passant.apply_action(&ChessMove::new(
            Square::make_square(Rank::Fifth, File::E), 
            Square::make_square(Rank::Sixth, File::F),
            None
        ));
        assert!(test.board == ground_truth);
    }

    /// Test to see if promotion is handled correctly.
    #[test]
    fn test_apply_action_new_before_promotion() {
        let before_promotion = ChessState::from_str(
            "rnbq1bnr/ppppkP1p/4p1p1/8/8/8/PPPP1PPP/RNBQKBNR w KQ - 1 5".to_string()
        );

        let ground_truth = Board::from_str(
            "rnbq1bQr/ppppk2p/4p1p1/8/8/8/PPPP1PPP/RNBQKBNR b KQ - 0 5"
        ).unwrap();
        let test = before_promotion.apply_action(&ChessMove::new(
            Square::make_square(Rank::Seventh, File::F), 
            Square::make_square(Rank::Eighth, File::G),
            Some(Piece::Queen)
        ));
        assert!(test.board == ground_truth);

        let ground_truth = Board::from_str(
            "rnbq1bNr/ppppk2p/4p1p1/8/8/8/PPPP1PPP/RNBQKBNR b KQ - 0 5"
        ).unwrap();
        let test = before_promotion.apply_action(&ChessMove::new(
            Square::make_square(Rank::Seventh, File::F), 
            Square::make_square(Rank::Eighth, File::G),
            Some(Piece::Knight)
        ));
        assert!(test.board == ground_truth);
    }

    /// Tests to check that legal move lists are correct and complete.

    /// Tests legal move generation during an abitrarily middle game position.
    #[test]
    fn test_generate_legal_actions_from_middle_game() {
        let middle_game = ChessState::from_str(
            "rnb1kbnr/ppp2ppp/3p4/4p1q1/4P3/3P1N2/PPP2PPP/RNBQKB1R w KQkq - 1 4".to_string()
        );
        
        let mut ground_truth = [
            // Pawn moves.
            ChessMove::from_san(&middle_game.board, "a3").unwrap(),
            ChessMove::from_san(&middle_game.board, "a4").unwrap(),
        
            ChessMove::from_san(&middle_game.board, "b3").unwrap(),
            ChessMove::from_san(&middle_game.board, "b4").unwrap(),
            
            ChessMove::from_san(&middle_game.board, "c3").unwrap(),
            ChessMove::from_san(&middle_game.board, "c4").unwrap(),
            
            ChessMove::from_san(&middle_game.board, "d4").unwrap(),
            
            ChessMove::from_san(&middle_game.board, "g3").unwrap(),
            ChessMove::from_san(&middle_game.board, "g4").unwrap(),
            
            ChessMove::from_san(&middle_game.board, "h3").unwrap(),
            ChessMove::from_san(&middle_game.board, "h4").unwrap(),

            // Knight moves.
            ChessMove::from_san(&middle_game.board, "Na3").unwrap(),
            ChessMove::from_san(&middle_game.board, "Nc3").unwrap(),
            ChessMove::from_san(&middle_game.board, "Nbd2").unwrap(),
            
            ChessMove::from_san(&middle_game.board, "Nfd2").unwrap(),
            ChessMove::from_san(&middle_game.board, "Nd4").unwrap(),
            ChessMove::from_san(&middle_game.board, "Nxe5").unwrap(),
            ChessMove::from_san(&middle_game.board, "Nxg5").unwrap(),
            ChessMove::from_san(&middle_game.board, "Nh4").unwrap(),
            ChessMove::from_san(&middle_game.board, "Ng1").unwrap(),

            // Bishop moves.
            ChessMove::from_san(&middle_game.board, "Be2").unwrap(),
            
            ChessMove::from_san(&middle_game.board, "Bd2").unwrap(),
            ChessMove::from_san(&middle_game.board, "Be3").unwrap(),
            ChessMove::from_san(&middle_game.board, "Bf4").unwrap(),
            ChessMove::from_san(&middle_game.board, "Bxg5").unwrap(),

            // Rook moves.
            ChessMove::from_san(&middle_game.board, "Rg1").unwrap(),

            // Queen moves.
            ChessMove::from_san(&middle_game.board, "Qd2").unwrap(),
            ChessMove::from_san(&middle_game.board, "Qe2").unwrap(),

            // King moves.
            ChessMove::from_san(&middle_game.board, "Ke2").unwrap(),
        ];
        ground_truth.sort();

        let mut test = middle_game.generate_legal_actions();
        test.sort();

        assert!(test == ground_truth);
    }

    /// Test to see if legal move generation is correct while in check.
    #[test]
    fn test_generate_legal_actions_during_check() {
        let during_check = ChessState::from_str(
            "rnb1kbnr/pppp1ppp/8/4P3/7q/8/PPPPP1PP/RNBQKBNR w KQkq - 1 3".to_string()
        );

        let mut ground_truth = [ChessMove::from_san(&during_check.board, "g3").unwrap()];
        ground_truth.sort();
        
        let mut test = during_check.generate_legal_actions();
        test.sort();
        
        assert!(test == ground_truth);
    }

    /// Ensures legal move list is empty after checkmate.
    #[test]
    fn test_generate_legal_actions_after_mate() {
        let after_mate = ChessState::from_str(
            "rnb1kbnr/pppp1ppp/4p3/8/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3".to_string()
        );
        assert!(after_mate.generate_legal_actions().len() == 0);
    }

    /// Test whether current player indicator is correct.
    #[test]
    fn test_side_to_move() {
        let game_start = ChessState::from_str(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
        );
        assert!(game_start.side_to_move());

        let middlegame_black_to_move = ChessState::from_str(
            "rn2kbnr/ppp3pp/3q1p2/4p3/4P1b1/3P1P2/PPP3PP/RNBQK2R b KQkq - 0 7".to_string()
        );
        assert!(!middlegame_black_to_move.side_to_move());
        
        let checkmate_white_to_move = ChessState::from_str(
            "rn2k1nr/ppp3pp/5p2/2b1p3/4P3/3P3P/PPP4K/RNB3q1 w kq - 4 19".to_string()
        );
        assert!(checkmate_white_to_move.side_to_move());
    }
}
