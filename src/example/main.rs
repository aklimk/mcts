#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use chess::{Board, ChessMove, BoardStatus};
use mcts::mcts::MCTSTree;
use mcts::chess_env::ChessState;
use std::io;
use std::io::{stdin, Write};
use std::str::FromStr;


/// Returns a symbolized char version of piece.
fn piece_to_char(piece: chess::Piece) -> char {
    match piece {
        chess::Piece::Pawn => return 'p',
        chess::Piece::Knight => return 'n',
        chess::Piece::Bishop => return 'b',
        chess::Piece::Rook => return 'r',
        chess::Piece::Queen => return 'q',
        chess::Piece::King => return 'k',
    }
}


/// Print chess position to stdout.
fn print_board(board: Board) {
    for i in (0..8).rev() {
        for j in 0..8 {
            // Convert numerical file to alphabetical one by adding the 
            // unicode `a` and converting back to char. This works becouse
            // lowercase letters are refferenced sequentially in unicode.
            let mut square: String = char::from_u32(
                0x61 + j
            ).unwrap().to_string();
            square += &(i + 1).to_string(); // The rank is 1 indexed.

            // If there is no piece, color defaults to white.
            let color = board.color_on(chess::Square::from_str(&square).unwrap())
                .unwrap_or(chess::Color::White);
            let piece = board.piece_on(chess::Square::from_str(&square).unwrap());
            if piece.is_some() {
                // Pint white pieces in uppercase, black pieces in lowercase.
                if color == chess::Color::White {
                    print!("{} ", piece_to_char(piece.unwrap()).to_uppercase());
                }
                else {
                    print!("{} ", piece_to_char(piece.unwrap()));
                }
            }
            // Emulates an empty square with no piece on it.
            else {
                print!("  ");
            }
        }
        println!("");
    }    
    println!("\n");
}


/// Provides a basic match against the MCTS engine in chess.
/// User always goes first.
pub fn main() {
    let runs = 50000;
    let mut game_state = Board::default();
    
    while game_state.status() == BoardStatus::Ongoing {
        // Get user move in SAN.
        let mut move_text = String::new();
        print!("Enter a SAN move: ");
        let _ = io::stdout().flush();
        let _ = stdin().read_line(&mut move_text);

        // Make move and update gamestate.
        game_state = game_state.make_move_new(ChessMove::from_san(&game_state, &move_text).unwrap());

        // Print board after user move.
        print_board(game_state);


        // Construct MCTS tree from game state fen and find optimal path.
        let mut tree = MCTSTree::<ChessMove, ChessState>::with_capacity(100000, None, game_state.to_string(), 30);
        for _i in 0..runs {
            let select = tree.select(0, None);
            let expand = tree.expand(select);
            let simulate = tree.simulate(expand);
            tree.backpropagate(expand, simulate);
        }
        let path = tree.trace_path(tree.select(0, Some(0.0)));

        // Best move is first move of optimal path.
        let action = &tree.arena[path[0]];

        // Make optimal move.
        game_state = game_state.make_move_new(action.game_state.last_move.unwrap());

        // Print board after MCTS move.
        print_board(game_state);
    }
}
