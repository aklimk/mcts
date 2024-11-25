/// Represents the possible outcomes of a two player, turn-based game.
///
/// This is used to logically abstract results and their data representation.
///
/// PartialEq is used for comparison, while Debug is used for printing results.
#[derive(PartialEq, Debug)]
pub enum GameResult {
    FirstPlayerWin,
    SecondPlayerWin,
    Draw,
}


/// Represents the required game state behaviour neccecary for 
/// MCTS to sucessfully generate, explore and debug game state trees.
///
/// Action is a generic data type representing a single legal move in the game.
pub trait GameState<Action> {
    
    /// Parses a gamestate from a string representation.
    fn from_str(game_state: String) -> Self;
    
    /// Generates a gamestate copy with `action` applied to it.
    fn apply_action(&self, action: &Action) -> Self;
    
    /// Determines whether the game has ended, given the fact that there
    /// are still legal moves left.
    ///
    /// # Invariants
    /// Assumes that there still are legal moves to be played.
    ///
    /// # Returns
    /// True if the game has ended, false otherwise.
    fn status_with_moves_left(&self) -> bool;

    /// Determines who has won the game, assuming that the game has ended.
    ///
    /// # Invariants
    /// Assumes that the game has ended.
    ///
    /// # Returns
    /// Game result with the result of the game.
    fn result(&self) -> GameResult;

    /// Generates possible legal actions from the current position.
    fn generate_legal_actions(&self) -> Vec<Action>;

    /// Determines the side that is due to move.
    ///
    /// Some gamestates are identical, except that the opposite player must move. 
    /// This function allows for the differentiation of thoose states.
    ///
    /// # Returns
    /// True if the first player is due to move, 
    /// false if the second player is due to move.
    fn side_to_move(&self) -> bool;
}
