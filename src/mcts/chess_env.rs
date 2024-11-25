use crate::game_state_trait::{GameResult, GameState};
use chess::{Board, ChessMove, Color, MoveGen, Piece};
use std::str::FromStr;

/// Represents the state of the chess board at a specific point in time.
/// Wraps the Chess `Board` with features to allow for the 50-move rule
/// and move histories by tracking the last move made.
#[derive(Debug, Clone)]
pub struct ChessState {
    pub board: Board,
    pub stalemate_50_move_counter: u16,
    pub last_move: Option<ChessMove>,
    pub depth: u32
}


/// Implementation of the GameState trait for the chess environment.
impl GameState<ChessMove> for ChessState {
    #[inline(always)]
    fn from_str(starting_fen: String) -> Self {
        return ChessState {board: Board::from_str(&starting_fen).expect("starting fen invalid"), stalemate_50_move_counter: 0, last_move: None, depth: 0};
    }


    #[inline(always)]
    fn apply_action(&self, action: &ChessMove) -> Self {
        let mut new_stalemate_50_move_counter = self.stalemate_50_move_counter + 1;
        // Check for conditions that reset the 50-move counter. 
        // This includes all captures and any pawn moves.
        let check1 = self.board.piece_on(action.get_dest()).is_some();
        let src_piece = self.board.piece_on(action.get_source());
        let check2 = src_piece.is_some() && src_piece.unwrap() == Piece::Pawn;

        if check1 || check2 {
            new_stalemate_50_move_counter = 0;
        }
        return ChessState {board: self.board.make_move_new(*action), stalemate_50_move_counter: new_stalemate_50_move_counter, last_move: Some(*action), depth: self.depth + 1};
    }

    #[inline(always)]
    fn status_with_moves_left(&self) -> bool {
        if self.stalemate_50_move_counter >= 50 {
            return false;
        }
        return true;
    }

    #[inline(always)]
    fn result(&self) -> GameResult {
        if self.board.checkers() == &chess::EMPTY || self.stalemate_50_move_counter >= 50 {
            return GameResult::Draw;
        } 
        if self.board.side_to_move() == Color::Black {
            return GameResult::FirstPlayerWin;
        }
        else {
            return GameResult::SecondPlayerWin;
        }
    }

    #[inline(always)]
    fn generate_legal_actions(&self) -> Vec<ChessMove> {
        return MoveGen::new_legal(&self.board).collect();
    }

    #[inline(always)]
    fn side_to_move(&self) -> bool {
        return self.board.side_to_move() == Color::White;
    }
}



/// Defines unit tests for the chess environment.
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use chess::Piece;
    use chess::Square;
    use chess::Rank;
    use chess::File;

    /// Test if default root generation is correct.
    #[test]
    fn test_generate_root() {
        assert!(Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap() == ChessState::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()).board);
    }
    
    /// Tests to see if applying an action leads to the correct state.
    #[test]
    fn test_apply_action_new_from_start() {
        let starting_state = ChessState::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());

        let e4 = starting_state.apply_action(&ChessMove::new(
            Square::make_square(Rank::Second, File::E), 
            Square::make_square(Rank::Fourth, File::E),
            None
        ));
        assert!(e4.board == Board::from_str("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").unwrap());

        let nf3 = starting_state.apply_action(&ChessMove::new(
            Square::make_square(Rank::First, File::G), 
            Square::make_square(Rank::Third, File::F),
            None
        ));
        assert!(nf3.board == Board::from_str("rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1").unwrap());
    }
    
    #[test]
    fn test_apply_action_new_before_end() {
        let before_fools_mate = ChessState {board: Board::from_str("rnbqkbnr/pppp1ppp/4p3/8/5P2/4P3/PPPP2PP/RNBQKBNR b KQkq - 0 2").unwrap(), stalemate_50_move_counter: 0, last_move: None, depth: 0};

        let qh4 = before_fools_mate.apply_action(&ChessMove::new(
            Square::make_square(Rank::Eighth, File::D), 
            Square::make_square(Rank::Fourth, File::H),
            None
        ));
        assert!(qh4.board == Board::from_str("rnb1kbnr/pppp1ppp/4p3/8/5P1q/4P3/PPPP2PP/RNBQKBNR w KQkq - 1 3").unwrap());
    }
    
    #[test]
    fn test_apply_action_new_enpassant() {
        let before_en_passant = ChessState {board: Board::from_str("rnbqkbnr/pppp2pp/4p3/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3").unwrap(), stalemate_50_move_counter: 0, last_move: None, depth: 0};

        let pf6 = before_en_passant.apply_action(&ChessMove::new(
            Square::make_square(Rank::Fifth, File::E), 
            Square::make_square(Rank::Sixth, File::F),
            None
        ));

        assert!(pf6.board == Board::from_str("rnbqkbnr/pppp2pp/4pP2/8/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 3").unwrap());
    }

    #[test]
    fn test_apply_action_new_before_promotion() {
        let before_promotion = ChessState {board: Board::from_str("rnbq1bnr/ppppkP1p/4p1p1/8/8/8/PPPP1PPP/RNBQKBNR w KQ - 1 5").unwrap(), stalemate_50_move_counter: 0, last_move: None, depth: 0};

        let pg8_q = before_promotion.apply_action(&ChessMove::new(
            Square::make_square(Rank::Seventh, File::F), 
            Square::make_square(Rank::Eighth, File::G),
            Some(Piece::Queen)
        ));
        assert!(pg8_q.board == Board::from_str("rnbq1bQr/ppppk2p/4p1p1/8/8/8/PPPP1PPP/RNBQKBNR b KQ - 0 5").unwrap());

        let pg8_n = before_promotion.apply_action(&ChessMove::new(
            Square::make_square(Rank::Seventh, File::F), 
            Square::make_square(Rank::Eighth, File::G),
            Some(Piece::Knight)
        ));
        assert!(pg8_n.board == Board::from_str("rnbq1bNr/ppppk2p/4p1p1/8/8/8/PPPP1PPP/RNBQKBNR b KQ - 0 5").unwrap());
    }

    /// Tests to check that legal move lists are correct and complete.
    #[test]
    fn test_generate_legal_actions_from_middle_game() {
        let middle_game = ChessState {board: Board::from_str("rnb1kbnr/ppp2ppp/3p4/4p1q1/4P3/3P1N2/PPP2PPP/RNBQKB1R w KQkq - 1 4").unwrap(), stalemate_50_move_counter: 0, last_move: None, depth: 0};
        let mut legal_moves: Vec<ChessMove> = middle_game.generate_legal_actions();
        legal_moves.sort();
        let mut ground_truth = [
            ChessMove::new(Square::make_square(Rank::Second, File::A), Square::make_square(Rank::Third, File::A), None),
            ChessMove::new(Square::make_square(Rank::Second, File::B), Square::make_square(Rank::Third, File::B), None),
            ChessMove::new(Square::make_square(Rank::Second, File::C), Square::make_square(Rank::Third, File::C), None),
            ChessMove::new(Square::make_square(Rank::Second, File::G), Square::make_square(Rank::Third, File::G), None),
            ChessMove::new(Square::make_square(Rank::Second, File::H), Square::make_square(Rank::Third, File::H), None),
            
            ChessMove::new(Square::make_square(Rank::Second, File::A), Square::make_square(Rank::Fourth, File::A), None),
            ChessMove::new(Square::make_square(Rank::Second, File::B), Square::make_square(Rank::Fourth, File::B), None),
            ChessMove::new(Square::make_square(Rank::Second, File::C), Square::make_square(Rank::Fourth, File::C), None),
            ChessMove::new(Square::make_square(Rank::Second, File::G), Square::make_square(Rank::Fourth, File::G), None),
            ChessMove::new(Square::make_square(Rank::Second, File::H), Square::make_square(Rank::Fourth, File::H), None),

            ChessMove::new(Square::make_square(Rank::Third, File::D), Square::make_square(Rank::Fourth, File::D), None),


            ChessMove::new(Square::make_square(Rank::First, File::B), Square::make_square(Rank::Third, File::A), None),
            ChessMove::new(Square::make_square(Rank::First, File::B), Square::make_square(Rank::Third, File::C), None),
            ChessMove::new(Square::make_square(Rank::First, File::B), Square::make_square(Rank::Second, File::D), None),

            ChessMove::new(Square::make_square(Rank::First, File::C), Square::make_square(Rank::Second, File::D), None),
            ChessMove::new(Square::make_square(Rank::First, File::C), Square::make_square(Rank::Third, File::E), None),
            ChessMove::new(Square::make_square(Rank::First, File::C), Square::make_square(Rank::Fourth, File::F), None),
            ChessMove::new(Square::make_square(Rank::First, File::C), Square::make_square(Rank::Fifth, File::G), None),

            ChessMove::new(Square::make_square(Rank::First, File::D), Square::make_square(Rank::Second, File::D), None),
            ChessMove::new(Square::make_square(Rank::First, File::D), Square::make_square(Rank::Second, File::E), None),

            ChessMove::new(Square::make_square(Rank::First, File::E), Square::make_square(Rank::Second, File::E), None),

            ChessMove::new(Square::make_square(Rank::First, File::F), Square::make_square(Rank::Second, File::E), None),

            ChessMove::new(Square::make_square(Rank::Third, File::F), Square::make_square(Rank::Second, File::D), None),
            ChessMove::new(Square::make_square(Rank::Third, File::F), Square::make_square(Rank::Fourth, File::D), None),
            ChessMove::new(Square::make_square(Rank::Third, File::F), Square::make_square(Rank::Fifth, File::E), None),
            ChessMove::new(Square::make_square(Rank::Third, File::F), Square::make_square(Rank::First, File::G), None),
            ChessMove::new(Square::make_square(Rank::Third, File::F), Square::make_square(Rank::Fifth, File::G), None),
            ChessMove::new(Square::make_square(Rank::Third, File::F), Square::make_square(Rank::Fourth, File::H), None),

            ChessMove::new(Square::make_square(Rank::First, File::H), Square::make_square(Rank::First, File::G), None)
        ];
        ground_truth.sort();

        assert!(legal_moves == ground_truth);
    }

    #[test]
    fn test_generate_legal_actions_during_check() {
        let during_check = ChessState {board: Board::from_str("rnb1kbnr/pppp1ppp/8/4P3/7q/8/PPPPP1PP/RNBQKBNR w KQkq - 1 3").unwrap(), stalemate_50_move_counter: 0, last_move: None, depth: 0};
        assert!(during_check.generate_legal_actions() == [
            ChessMove::new(
                Square::make_square(Rank::Second, File::G), 
                Square::make_square(Rank::Third, File::G), 
                None
            )
        ]);
    }

    #[test]
    fn test_generate_legal_actions_after_mate() {
        let after_mate = ChessState {board: Board::from_str("rnb1kbnr/pppp1ppp/4p3/8/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3").unwrap(), stalemate_50_move_counter: 0, last_move: None, depth: 0};
        assert!(after_mate.generate_legal_actions() == []);
    }

    /// Test whether current player turn switch is correct.
    #[test]
    fn test_side_to_move() {
        let game_start = ChessState {board: Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap(), stalemate_50_move_counter: 0, last_move: None, depth: 0};
        assert!(game_start.side_to_move());

        let middlegame_black_to_move = ChessState {board: Board::from_str("rn2kbnr/ppp3pp/3q1p2/4p3/4P1b1/3P1P2/PPP3PP/RNBQK2R b KQkq - 0 7").unwrap(), stalemate_50_move_counter: 0, last_move: None, depth: 0};
        assert!(!middlegame_black_to_move.side_to_move());
        
        let checkmate_white_to_move = ChessState {board: Board::from_str("rn2k1nr/ppp3pp/5p2/2b1p3/4P3/3P3P/PPP4K/RNB3q1 w kq - 4 19").unwrap(), stalemate_50_move_counter: 0, last_move: None, depth: 0};
        assert!(checkmate_white_to_move.side_to_move());
    }
}
