# mcts
Includes a fully featured MCTS engine library with random rollouts, with corresponding unit tests. 
The library can be applied to any game by simply creating a `GameState` trait for a particular game.
Includes an example chess `GameState` implementation unit/integration tests and puzzles to solve.

# Features
- Core MCTS engine is fully game-agnostic with no domain knowledge and random rollouts.
    Implement the included `GameState` trait and the engine will operate on any game.
- Includes fully featured unit and integration tests for both the fundemental library and chess implementation.
- Uses arena based memory allocation for nodes. Avoids self-referencial node data structures and explicit pointers.
- MCTS engine uses fast rust memory allocation library as well as fast seed-based random number generation library.
- Includes fully function MCTS algorithm with selection, expansion, simulation/rollout and backpropagation.
- Selection uses the UCT algorithm to deliver a theoretically perfect balance between explotation and exploration, with
    customizable exploration factor.

# Building/Running
The project is packaged as a single rust library crate, with a chess example packaged as an example binary.

To build the library and corresponding example, simply use ```cargo build --release```.

To run the unit/integration tests and chess puzzle solver, simply run ```cargo test --release```.
