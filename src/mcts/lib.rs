//! # Monte Carlo Tree Search (MCTS) Implemented in Rust
//! 
//! Mcts is a method of finding optimal paths within a tree, given a non deterministic
//! heuristic function. It estimates the proprtion of time neccecary to spend on 
//! exploration vs exploitation.
//!
//! Mcts involves:
//! Selection: The tree is traversed and the path with maximum UCT is choosen.
//! Expansion: A selected node with unexpanded children is randomly expanded.
//! Simulation: The game is simulated from the current node.
//! Backpropagation: The result statistics applies to the entre path upto the root.
//!
//! This project provides a mcts engine that uses random simulations/rollouts 
//! in order to explore constructed game trees using the UCT algorithm. 
//! 
//! The GameState trait specifies the functions and types neccecary for the engine
//! to be able to generate trees. 
//!
//! The monte carlo tree search structure and implementation provides utilities to
//! perfom the 4 mcts stages on a given game representation. as well as unit tests 
//! for each component.

pub mod game_state_trait;
pub mod mcts;
pub mod chess_env;