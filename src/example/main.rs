#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
use chess::{Board, ChessMove};
use mcts::mcts::MCTSTree;
use std::io::stdin;
use mcts::chess_env::ChessState;
use std::io;
use std::io::Write;
use std::str::FromStr;

// Convert chess piece to cli notation.
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

// Print current board state to cli.
fn print_board(board: Board) {
    for i in (0..8).rev() {
        for j in 0..8 {
            let square: String = char::from_u32(0x61 + j).unwrap().to_string() + &(i + 1).to_string();
            let color = board.color_on(chess::Square::from_str(&square).unwrap()).unwrap_or(chess::Color::White);
            let piece = board.piece_on(chess::Square::from_str(&square).unwrap());
            if piece.is_some() {
                if color == chess::Color::White {
                    print!("{} ", piece_to_char(piece.unwrap()).to_uppercase());
                }
                else {
                    print!("{} ", piece_to_char(piece.unwrap()));
                }
            }
            else {
                print!("  ");
            }
        }
        println!("");
    }    
    println!("\n");
}

pub fn main() {
    let runs = 50000;
    let mut game_state = Board::default();
    
    loop {
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
