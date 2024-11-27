use crate::game_state_trait::GameState;
use crate::game_state_trait::GameResult;

// Psuedorandom selection is used for simualtions/rollouts. Xorshfit is not cryptographically 
// secure and less random than other implementations, but very fast.
use xorshift::{Rng, SeedableRng, Xorshift128};

/// Represents a node in the mcts game tree. 
/// It holds game tree information as well as mcts statistics.
///
/// Trees datastructures in general hold their parent as well as a list of thier children.
///
/// Game trees associate each node with a gamestate and each line as an action.
///
/// Mcts progressively builds the tree, starting with children being represented as an action before being explored
/// to having a full node representation once explored.
///
/// Results of rollouts and child rollouts are kept within the tree for ucb calculation.
///
/// In order to avoid self referential structure sizing, all references to nodes are 
/// opaque pointers, with the actual nodes being allocated within a memory arena.
pub struct MCTSNode<Action, GameStateObj> 
where
    GameStateObj: GameState<Action> + Clone
{
    // Game state asocciated with the node in the tree.
    pub game_state: GameStateObj,
    
    /// Parent of current node. None if root, as root has no parent.
    pub parent: Option<usize>,
    
    /// Tree indexes for already expanded children.
    pub expanded: Vec<usize>,
    
    /// Legal moves corresponding to unexpanded child nodes.
    pub unexpanded: Vec<Action>,
    
    /// Sum of all simulation wins of the sub-graph with the current node as its root.
    pub wins: u32,
    
    /// Sum of all simulation draws of the sub-graph with the current node as its root.
    /// technically not needed for MCTS, but allows for differentiating draws and losses.
    pub draws: u32,
    
    /// Sum of all simulations of the sub-graph with the current node as its root.
    pub sims: u32,
}


/// Holds the node memory arena for the mcts tree and 
/// associated mcts tree properties.
pub struct MCTSTree<Action, GameStateObj> 
where
    GameStateObj: GameState<Action> + Clone
{
    /// Memory arena for mcts nodes.
    pub arena: Vec<MCTSNode<Action, GameStateObj>>,

    /// Average children expected for nodes in the tree.
    ///
    /// Used to create reasnobly sized vectors and avoid unneccecary,
    /// allocations at the expense of extra used memory.
    pub average_child_count: usize,

    /// Holds the current random generator state. Random numbers will be generated
    /// from the current state and the state will be modified.
    pub random_generator: Xorshift128,
}


/// Methods to enable the creation, search and expansion of the MCTSTree.
/// Based on the supplied game state methods.
impl<Action, GameStateObj> MCTSTree<Action, GameStateObj> 
where
    GameStateObj: GameState<Action> + Clone
{
    /// Creates a new mcts tree with an arena capacity, starting seed and position.
    ///
    /// # Arguments
    /// * `capacity` : The starting size of the memory arena. Larger values trade increase
    /// memory usage for less dynamic allocation of new memory.
    ///
    /// * `seed` : The seed that determines the starting state of the rng.
    ///
    /// * `starting_pos` : String encoding the starting position of the game.
    ///
    /// * `average_child_count` : Number of children expected for nodes in the tree.
    pub fn with_capacity(
        arena_capacity: usize, 
        seed: Option<u64>, 
        starting_pos: String, 
        average_child_count: usize)
    -> Self {
        // Initilize the tree data structures.
        // Seed is a 128 bit number, or a slice of 2 64 bit ones,
        // this method expands the 64 bit seed to 128 bit.
        let seed_formatted: &[_] = &[seed.unwrap_or(0), 0];
        
        let mut tree = Self {
            arena: Vec::with_capacity(arena_capacity), 
            average_child_count: average_child_count,
            random_generator: SeedableRng::from_seed(seed_formatted), 
        };

        // Create the root node of the tree.
        let root_game_state = GameStateObj::from_str(starting_pos);
        let unexpanded = root_game_state.generate_legal_actions();
        tree.arena.push(MCTSNode {
            game_state: root_game_state, 
            parent: None, 
            expanded: Vec::with_capacity(tree.average_child_count), 
            unexpanded: unexpanded,
            wins: 0, draws: 0, sims: 0
        });
        
        return tree;
    }

    /// Implementation of the UCT algorithm for a particular node.
    ///
    /// # Arguments
    /// * `exploration_factor` : corresponds to `c` in the UCT algorithm, 
    /// a higher exploration_factor means a preference towards exploration over exploitation. 
    /// Sqrt(2) is the theoretical optimum and default if unspecified.
    /// 
    /// # Returns
    /// UCT value associated with the selected node and tree.
    /// 
    /// # Panics
    /// If child_index has no parent, the method will panic on unwrap.
    /// A parent is required as it is part of the UCT algorithm.
    pub fn uct(&self, child: usize, exploration_factor: Option<f32>) -> f32 {
        
        let child_obj = &self.arena[child];
        
        // Parent must be specified.
        child_obj.parent.expect("no parent");
        
        let wins = child_obj.wins as f32;
        let sims = child_obj.sims as f32;
        let parent_sims = self.arena[child_obj.parent.unwrap()].sims as f32;

        // UCT = (wins / sims) + c*sqrt(ln(parent_sims) / sims).
        return (wins / sims) + exploration_factor.unwrap_or(f32::sqrt(2.0)) * f32::sqrt(f32::ln(parent_sims) / sims);
    }

    /// Returns the child node of `parent` with the maximum uct.
    ///
    /// # Arguments
    /// * `parent` : Parent to search the children of.
    ///
    /// * `exploration_factor` : Corresponds to `c` in the UCT algorithm, 
    /// a higher exploration_factor means a preference to exploration over exploitation. 
    /// Sqrt(2) is the theoretical optimum and is the default if unspecified.
    pub fn get_max_uct_child(&self, parent: usize, exploration_factor: Option<f32>) -> usize {
        let mut best_value: f32 = f32::MIN;
        let mut best_child: usize = 0;
        for child in &self.arena[parent].expanded {
            // If the child has a greater uct than the previous maximum,
            // replace the maximum with the current child.
            let child_uct = self.uct(*child, exploration_factor);
            if child_uct > best_value {
                best_value = child_uct;
                best_child = *child;
            }
        }
        return best_child;
    }

    /// Finds the optimal leaf node index in the MCTS Tree according to the path with maximal UCT at each depth.
    /// A leaf node is a node with unexpanded children, or a terminal node.
    ///
    /// # Arguments
    /// * `root_index` : The index of the MCTSTree vector to begin selection from.
    /// 
    /// * `exploration_factor` : Corresponds to `c` in the UCT algorithm, 
    /// a higher exploration_factor means a preference to exploration over exploitation. 
    /// Sqrt(2) is the theoretical optimum and is the default if unspecified.
    pub fn select(&self, mut root: usize, exploration_factor: Option<f32>) -> usize {
        // Leaf node is found where unexpanded children exist.
        while self.arena[root].unexpanded.len() == 0 {
            // If both expanded and unexpanded children are empty the node must be terminal and therefore a leaf node.
            if self.arena[root].expanded.len() == 0 {
                return root;
            }
            
            // Replace the root index with the expanded child with maximal UCT.
            root = self.get_max_uct_child(root, exploration_factor);
        }
        return root;
    }

    /// Expands a random unexpanded action from `leaf_node` returning its arena pointer.
    /// If the leaf node is terminal, no nodes are expanded and the leaf index is returned.
    ///
    /// # Invariants
    /// The leaf node is assumed to either have unexpanded children or be a terminal node.
    /// 
    /// # Arguments
    /// * `leaf_node` : The leaf node to expand a node on.
    ///
    /// # Returns
    /// A pointer to the newly expanded node, or `leaf_node` if the leaf node is terminal.
    pub fn expand(&mut self, leaf_node: usize) -> usize {
        // Return leaf node if its terminal.
        if self.arena[leaf_node].unexpanded.len() == 0 {
            return leaf_node;
        }
        
        // Select a random action from potential legal actions.
        let random_number = self.random_generator.gen_range(0, self.arena[leaf_node].unexpanded.len());
        let random_action = &self.arena[leaf_node].unexpanded[random_number];

        // Generate resulting game state after random action is applied;
        let expanded_game_state = self.arena[leaf_node].game_state.apply_action(random_action);

        // Generate possible actions.
        let expanded_game_state_unexpanded = expanded_game_state.generate_legal_actions();

        // Push new node to arena.
        self.arena.push(MCTSNode { 
            game_state: 
            expanded_game_state, 
            parent: Some(leaf_node), 
            expanded: Vec::with_capacity(self.average_child_count), 
            unexpanded: expanded_game_state_unexpanded, 
            wins: 0, draws: 0, sims: 0 
        });
        let expanded_node = self.arena.len() - 1;

        // Remove node from unexpanded and add to expanded.
        self.arena[leaf_node].unexpanded.remove(random_number);
        self.arena[leaf_node].expanded.push(expanded_node);

        return expanded_node;
    }

    /// Randomly selects possible moves for both players 
    /// until a terminal state is reached.
    /// returns the result as a GameResult.
    ///
    /// # Arguments
    /// * `node` : The node to start simulating from.
    ///
    /// # Returns
    /// The outcome of the random rollout.
    pub fn simulate(&mut self, node: usize) -> GameResult {
        let mut count = 0;
        let mut game_state = self.arena[node].game_state.clone();
        let mut actions = game_state.generate_legal_actions();
        while actions.len() > 0 && game_state.status_with_moves_left() {
            // Games are hard-capped to 200 moves.
            if count > 200 {
                return GameResult::Draw;
            }
            
            // Choose random action and replace the state with it.
            let random_number = self.random_generator.gen_range(0, actions.len());
            game_state = game_state.apply_action(&actions[random_number]);
            actions = game_state.generate_legal_actions();
            count += 1;
        }
        return game_state.result();
    }

    /// Backpropagates a game result up the tree, starting at node index.
    ///
    /// For every node propagated, if the node is the same side as the winning side, one is added to wins,
    /// otherwise one is added to either draws or nothing.
    /// In either case one is added to simulations.
    ///
    /// # Arguments
    /// * `current_node` : The current node that is being backpropagated.
    ///
    /// * `result` : The result of the simulation that is being backpropagated against.
    pub fn backpropagate(&mut self, mut current_node: usize, result: GameResult) {
        loop {
            let current_node_object = &mut self.arena[current_node];

            if result == GameResult::Draw {
                current_node_object.draws += 1;
            }
            
            // There is a winning player.
            else {
                // Converts the result into a bool where true indicates the first player has
                // won and false indicates the second player has won. This is compared to 
                // the side due to move.
                // Note: First player being due to move means that it is the second players turn.
                let result_bool = result == GameResult::FirstPlayerWin;
                let side_bool = !current_node_object.game_state.side_to_move();
                if result_bool == side_bool {
                    current_node_object.wins += 1;
                }
            }

            // A simulation count is added for every node that is backpropagated.
            current_node_object.sims += 1;

            // Stop backpropagated if the root node is reached.
            if current_node_object.parent.is_none() {
                break;
            }

            // The current node becomes the parent.
            current_node = current_node_object.parent.expect("no parent");
        }
    }

    /// Gives the list of actions that leads to a specific leaf node in the tree
    /// from `current_node`.
    ///
    /// The action taken to get to the current state is not included in the path.
    #[inline(always)]
    pub fn trace_path(&self, mut current_node: usize) -> Vec<usize> {
        let mut path: Vec<usize> = Vec::new();
        
        // Constructs the path backwards, starting from the leaf node, and then reverses it.
        loop {
            // If the node has no parent, it is a root node and the path tracing has finished.
            if self.arena[current_node].parent.is_none() {
                break;
            }
            
            // Push current node to temporary path and replace 
            path.push(current_node);
            current_node = self.arena[current_node].parent.expect("no parent");
        }
        return path.into_iter().rev().collect();
    }
}



/// Unit tests for components of the MCTS tree.
#[cfg(test)]
mod tests {
    use super::*;

    /// Placeholder game-state which holds only basic internal logic
    /// it has the neccecary logic to test everything except for 
    /// the simulation/rollout function.
    #[derive(Debug, Clone)]
    struct PlaceHolderState {
        last_action_made: u16,
        depth_counter: u16
    }

    impl GameState<u16> for PlaceHolderState {
        fn from_str(_starting_fen: String) -> Self {
            return PlaceHolderState {last_action_made: 0, depth_counter: 0};
        }
        
        fn apply_action(&self, action: &u16) -> Self {
            return PlaceHolderState {
                last_action_made: *action, 
                depth_counter: self.depth_counter + 1
            };
        }
        
        fn status_with_moves_left(&self) -> bool {
            return false;
        }
        
        fn result(&self) -> GameResult {
            return GameResult::Draw;
        }
        
        fn generate_legal_actions(&self) -> Vec<u16> {
            return Vec::new();
        }
        
        fn side_to_move(&self) -> bool {
            return (self.depth_counter % 2) == 0;
        }
    }

    /// Generates MCTS sample tree for use during tests.
    /// Draws are ignored becouse they don't effect internal MCTS logic.
    fn test_generate_example_tree() -> MCTSTree<u16, PlaceHolderState> {
        // Abitrary tree parameters as PlaceHolderState largely ignores them.
        let mut tree = MCTSTree::<u16, PlaceHolderState>::with_capacity(
            100, 
            None, 
            "".to_string(), 
            10
        );
        tree.arena[0].wins = 5;
        tree.arena[0].sims = 12;
        tree.arena[0].expanded = vec![1, 8];
        // Left branch in example tree
        tree.arena.push(MCTSNode {
            game_state: PlaceHolderState {last_action_made: 0, depth_counter: 1}, 
            parent: Some(0), 
            expanded: vec![2, 4, 5], 
            unexpanded: Vec::new(), 
            wins: 5, draws: 0, sims: 8
        });
        // Left-Left branch in example tree.
        tree.arena.push(MCTSNode {
            game_state: PlaceHolderState {last_action_made: 0, depth_counter: 2}, 
            parent: Some(1), 
            expanded: vec![3], 
            unexpanded: Vec::new(), 
            wins: 1, draws: 0, sims: 2
        });        
        // Left-Left-Mid branch in example tree.
        tree.arena.push(MCTSNode {
            game_state: PlaceHolderState {last_action_made: 0, depth_counter: 3}, 
            parent: Some(2), 
            expanded: vec![], 
            unexpanded: vec![10, 11], 
            wins: 1, draws: 0, sims: 1
        });        
        // Left-Mid branch in example tree.
        tree.arena.push(MCTSNode {
            game_state: PlaceHolderState {last_action_made: 0, depth_counter: 2}, 
            parent: Some(1), 
            expanded: vec![], 
            unexpanded: Vec::new(), 
            wins: 0, draws: 0, sims: 1
        });     
        // Left-Right branch in example tree.
        tree.arena.push(MCTSNode {
            game_state: PlaceHolderState {last_action_made: 0, depth_counter: 2}, 
            parent: Some(1), 
            expanded: vec![6, 7], 
            unexpanded: Vec::new(), 
            wins: 2, draws: 0, sims: 4
        });     
        // Left-Right-Left branch in example tree.
        tree.arena.push(MCTSNode {
            game_state: PlaceHolderState {last_action_made: 0, depth_counter: 3}, 
            parent: Some(5), 
            expanded: vec![], 
            unexpanded: Vec::new(), 
            wins: 0, draws: 0, sims: 1
        });     
        // Left-Right-Right branch in example tree.
        tree.arena.push(MCTSNode {
            game_state: PlaceHolderState {last_action_made: 0, depth_counter: 3}, 
            parent: Some(5), 
            expanded: vec![], 
            unexpanded: Vec::new(), 
            wins: 2, draws: 0, sims: 2
        });     

        // Right branch in example tree.
        tree.arena.push(MCTSNode {
            game_state: PlaceHolderState {last_action_made: 0, depth_counter: 1}, 
            parent: Some(0), 
            expanded: vec![9, 10], 
            unexpanded: Vec::new(), 
            wins: 2, draws: 0, sims: 4
        });    
        // Right-Left in example tree.
        tree.arena.push(MCTSNode {
            game_state: PlaceHolderState {last_action_made: 0, depth_counter: 2}, 
            parent: Some(8), 
            expanded: vec![], 
            unexpanded: Vec::new(), 
            wins: 1, draws: 0, sims: 1
        });    
        // Right-Right in example tree.
        tree.arena.push(MCTSNode {
            game_state: PlaceHolderState {last_action_made: 0, depth_counter: 2}, 
            parent: Some(8), 
            expanded: vec![11], 
            unexpanded: Vec::new(), 
            wins: 1, draws: 0, sims: 2
        });    
        // Right-Right-Mid branch in example tree.
        tree.arena.push(MCTSNode {
            game_state: PlaceHolderState {last_action_made: 0, depth_counter: 3}, 
            parent: Some(10),
            expanded: vec![], 
            unexpanded: Vec::new(), 
            wins: 0, draws: 0, sims: 1
        });    

        return tree;
    }

    /// Running uct on root is invalid. Ensures unwrap on parent panics.
    #[test]
    #[should_panic]
    fn test_uct_root() {
        let tree = test_generate_example_tree();
        tree.uct(0, Some(f32::sqrt(2.0)));
    }

    /// Tests if the uct function generates the correct uct values.
    /// based on the example tree.
    #[test]
    fn test_uct() {
        let tree = test_generate_example_tree();
        assert!(format!("{:.3}", tree.uct(1, Some(f32::sqrt(2.0)))) == "1.413");
        assert!(format!("{:.3}", tree.uct(2, Some(f32::sqrt(2.0)))) == "1.942");
        assert!(format!("{:.3}", tree.uct(3, Some(f32::sqrt(2.0)))) == "2.177");
        assert!(format!("{:.3}", tree.uct(4, Some(f32::sqrt(2.0)))) == "2.039");
        assert!(format!("{:.3}", tree.uct(5, Some(f32::sqrt(2.0)))) == "1.520");
        assert!(format!("{:.3}", tree.uct(6, Some(f32::sqrt(2.0)))) == "1.665");
        assert!(format!("{:.3}", tree.uct(7, Some(f32::sqrt(2.0)))) == "2.177");
        assert!(format!("{:.3}", tree.uct(8, Some(f32::sqrt(2.0)))) == "1.615");
        assert!(format!("{:.3}", tree.uct(9, Some(f32::sqrt(2.0)))) == "2.665");
        assert!(format!("{:.3}", tree.uct(10,Some(f32::sqrt(2.0)))) == "1.677");
        assert!(format!("{:.3}", tree.uct(11,Some(f32::sqrt(2.0)))) == "1.177");
    }

    /// Tests if the select function selects the correct node from
    /// the tree based on the example tree.
    #[test]
    fn test_select() {
        let mut tree = test_generate_example_tree();
        println!("{}", tree.select(0, Some(f32::sqrt(2.0))));
        assert!(tree.select(0, Some(f32::sqrt(2.0))) == 9);
        tree.arena[1].wins = 8;
        assert!(tree.select(0, Some(f32::sqrt(2.0))) == 4);

    }

    /// Tests if the expansion function expands and reconfigures
    /// the nodes correctly.
    #[test]
    fn test_expand() {
        let mut tree = test_generate_example_tree();
        let i0 = tree.expand(3);
        let i1 = tree.expand(3);
        let i2 = tree.expand(3);
        assert!(tree.arena[i0].game_state.last_action_made == 11 || 
            tree.arena[i0].game_state.last_action_made == 10);
        if tree.arena[i0].game_state.last_action_made == 11 {
            assert!(tree.arena[i1].game_state.last_action_made == 10);
        }
        else {
            assert!(tree.arena[i1].game_state.last_action_made == 11);
        }
        assert!(tree.arena[i2].game_state.last_action_made == 0);
        assert!(tree.arena[3].expanded == (vec![i0, i1] as Vec<usize>));
        assert!(tree.arena[3].unexpanded.len() == 0);
    }

    /// Tests if the tree backpropagation correctly feeds 
    /// the simulation result to the nodes of the tree.
    #[test]
    fn test_backpropagate() {
        let mut tree = test_generate_example_tree();
        tree.backpropagate(0, GameResult::SecondPlayerWin);
        tree.backpropagate(6, GameResult::SecondPlayerWin);
        tree.backpropagate(10, GameResult::SecondPlayerWin);
        assert!(tree.arena[0].wins == 8 && tree.arena[0].sims == 15);
        assert!(tree.arena[1].wins == 5 && tree.arena[1].sims == 9);
        assert!(tree.arena[2].wins == 1 && tree.arena[2].sims == 2);
        assert!(tree.arena[3].wins == 1 && tree.arena[3].sims == 1);
        assert!(tree.arena[4].wins == 0 && tree.arena[4].sims == 1);
        assert!(tree.arena[5].wins == 3 && tree.arena[5].sims == 5);
        assert!(tree.arena[6].wins == 0 && tree.arena[6].sims == 2);
        assert!(tree.arena[7].wins == 2 && tree.arena[7].sims == 2);
        assert!(tree.arena[8].wins == 2 && tree.arena[8].sims == 5);
        assert!(tree.arena[9].wins == 1 && tree.arena[9].sims == 1);
        assert!(tree.arena[10].wins == 2 && tree.arena[10].sims == 3);
        assert!(tree.arena[11].wins == 0 && tree.arena[11].sims == 1);

        tree.backpropagate(11, GameResult::FirstPlayerWin);
        tree.backpropagate(3, GameResult::FirstPlayerWin);
        tree.backpropagate(1, GameResult::FirstPlayerWin);
        assert!(tree.arena[0].wins == 8 && tree.arena[0].sims == 18);
        assert!(tree.arena[1].wins == 7 && tree.arena[1].sims == 11);
        assert!(tree.arena[2].wins == 1 && tree.arena[2].sims == 3);
        assert!(tree.arena[3].wins == 2 && tree.arena[3].sims == 2);
        assert!(tree.arena[4].wins == 0 && tree.arena[4].sims == 1);
        assert!(tree.arena[5].wins == 3 && tree.arena[5].sims == 5);
        assert!(tree.arena[6].wins == 0 && tree.arena[6].sims == 2);
        assert!(tree.arena[7].wins == 2 && tree.arena[7].sims == 2);
        assert!(tree.arena[8].wins == 3 && tree.arena[8].sims == 6);
        assert!(tree.arena[9].wins == 1 && tree.arena[9].sims == 1);
        assert!(tree.arena[10].wins == 2 && tree.arena[10].sims == 4);
        assert!(tree.arena[11].wins == 1 && tree.arena[11].sims == 2);


        tree.backpropagate(4, GameResult::Draw);
        tree.backpropagate(7, GameResult::Draw);
        tree.backpropagate(9, GameResult::Draw);
        assert!(tree.arena[0].wins == 8 && tree.arena[0].sims == 21);
        assert!(tree.arena[1].wins == 7 && tree.arena[1].sims == 13);
        assert!(tree.arena[2].wins == 1 && tree.arena[2].sims == 3);
        assert!(tree.arena[3].wins == 2 && tree.arena[3].sims == 2);
        assert!(tree.arena[4].wins == 0 && tree.arena[4].sims == 2);
        assert!(tree.arena[5].wins == 3 && tree.arena[5].sims == 6);
        assert!(tree.arena[6].wins == 0 && tree.arena[6].sims == 2);
        assert!(tree.arena[7].wins == 2 && tree.arena[7].sims == 3);
        assert!(tree.arena[8].wins == 3 && tree.arena[8].sims == 7);
        assert!(tree.arena[9].wins == 1 && tree.arena[9].sims == 2);
        assert!(tree.arena[10].wins == 2 && tree.arena[10].sims == 4);
        assert!(tree.arena[11].wins == 1 && tree.arena[11].sims == 2);
    }
}
