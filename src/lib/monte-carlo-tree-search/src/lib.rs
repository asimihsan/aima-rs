/*
 * Copyright 2023 Asim Ihsan
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use rand::prelude::SliceRandom;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::time::{Duration, Instant};

use slotmap::new_key_type;

trait Action: Clone + Copy + PartialEq + Eq + Hash {}

trait State<_Action>: Clone + PartialEq + Eq + Hash
where
    _Action: Action,
{
    fn simulate(&self, playouts: Int) -> Vec<SimulationResult>;
    fn get_actions(&self) -> Vec<_Action>;
    fn get_next_state(&self, action: &_Action) -> Self;
}

type Int = i32;
type Float = f64;

new_key_type! { struct MctsNodeKey; }

#[derive(Debug, Clone)]
struct MctsNode<_State, _Action> {
    parent: Option<MctsNodeKey>,
    children: HashMap<_Action, MctsNodeKey>,
    visits: Int,
    wins: Int,
    state: Option<_State>,
}

impl<_State, _Action> MctsNode<_State, _Action> {
    fn new(parent: Option<MctsNodeKey>, state: Option<_State>) -> Self {
        Self {
            parent,
            children: HashMap::new(),
            visits: 0,
            wins: 0,
            state,
        }
    }
}

#[derive(Debug, Clone)]
struct MctsTree<_State, _Action> {
    nodes: slotmap::SlotMap<MctsNodeKey, MctsNode<_State, _Action>>,
    root: MctsNodeKey,
}

impl<_State, _Action> MctsTree<_State, _Action>
where
    _State: State<_Action>,
    _Action: Action,
{
    fn new(root_state: _State) -> Self {
        let mut nodes = slotmap::SlotMap::with_key();
        let root = nodes.insert(MctsNode::new(None, Some(root_state)));
        Self { nodes, root }
    }

    fn get_node_from_nodekey(&self, node: MctsNodeKey) -> &MctsNode<_State, _Action> {
        &self.nodes[node]
    }

    fn get_mut_node_from_nodekey(&mut self, node: MctsNodeKey) -> &mut MctsNode<_State, _Action> {
        &mut self.nodes[node]
    }

    fn get_root(&self) -> &MctsNode<_State, _Action> {
        &self.nodes[self.root]
    }

    fn get_mut_root(&mut self) -> &mut MctsNode<_State, _Action> {
        &mut self.nodes[self.root]
    }

    fn get_root_nodekey(&self) -> MctsNodeKey {
        self.root
    }

    fn get_children_nodekeys(&self, node: MctsNodeKey) -> &HashMap<_Action, MctsNodeKey> {
        &self.nodes[node].children
    }

    fn add_child(
        &mut self,
        state: Option<_State>,
        parent: MctsNodeKey,
        action: _Action,
    ) -> MctsNodeKey {
        let child = self.nodes.insert(MctsNode::new(Some(parent), state));
        self.nodes[parent].children.insert(action, child);
        child
    }
}

fn uct_score(
    node_visits: Int,
    node_wins: Int,
    parent_visits: Int,
    exploration_constant: Float,
) -> Float {
    let node_wins_float = Float::from(node_wins);
    let node_visits_float = Float::from(node_visits);
    let parent_visits_float = Float::from(parent_visits);
    let exploitation_term = node_wins_float / node_visits_float;
    let exploration_term =
        exploration_constant * (parent_visits_float.ln() / node_visits_float).sqrt();
    exploitation_term + exploration_term
}

fn uct_select<_State, _Action>(
    tree: &MctsTree<_State, _Action>,
    exploration_constant: Float,
) -> MctsNodeKey
where
    _State: State<_Action>,
    _Action: Action,
{
    let root = tree.get_root_nodekey();
    let mut node = root;
    loop {
        let children = tree.get_children_nodekeys(node);
        if children.is_empty() {
            break;
        }
        let parent_visits = tree.get_node_from_nodekey(node).visits;
        let (action, _best_child, _score) = children
            .iter()
            .map(|(action, child)| {
                let child_node = tree.get_node_from_nodekey(*child);
                let score = uct_score(
                    child_node.visits,
                    child_node.wins,
                    parent_visits,
                    exploration_constant,
                );
                (action, child, score)
            })
            .max_by(|(_, _, score1), (_, _, score2)| score1.partial_cmp(score2).unwrap())
            .unwrap();
        node = *children.get(action).unwrap();
    }
    node
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SimulationResult {
    Win,
    NotWin,
}

#[derive(Debug, Clone, Copy)]
pub enum IterationLimitKind {
    Iterations(usize),
    TimeSeconds(Duration),
}

// Mcts is the main Monte Carlo Tree Search algorithm.
// See section 5.4 Monte Carlo Tree Search page 162 and 163.
struct Mcts<_State, _Action> {
    tree: Rc<RefCell<MctsTree<_State, _Action>>>,
    iteration_limit: IterationLimitKind,
    exploration_constant: Float,
    playouts_per_simulation: Int,
}

impl<_State, _Action> Mcts<_State, _Action>
where
    _State: State<_Action>,
    _Action: Action,
{
    fn new(
        root_state: _State,
        iteration_limit: IterationLimitKind,
        exploration_constant: Float,
        playouts_per_simulation: Int,
    ) -> Self {
        Mcts::new_from_tree(
            MctsTree::new(root_state),
            iteration_limit,
            exploration_constant,
            playouts_per_simulation,
        )
    }

    // useful for tests
    fn new_from_tree(
        tree: MctsTree<_State, _Action>,
        iteration_limit: IterationLimitKind,
        exploration_constant: Float,
        playouts_per_simulation: Int,
    ) -> Self {
        Self {
            tree: Rc::new(RefCell::new(tree)),
            iteration_limit,
            exploration_constant,
            playouts_per_simulation,
        }
    }

    fn run(&mut self) {
        match self.iteration_limit {
            IterationLimitKind::Iterations(iterations) => {
                for _ in 0..iterations {
                    self.iteration();
                }
            }
            IterationLimitKind::TimeSeconds(time) => {
                let start = Instant::now();
                while start.elapsed() < time {
                    self.iteration();
                }
            }
        }
    }

    fn iteration(&mut self) {
        let node_key = self.select();
        let node_key = self.expand(node_key);
        let tree = Rc::clone(&self.tree);
        let tree = tree.borrow();
        let node = tree.get_node_from_nodekey(node_key);
        let result = node
            .state
            .as_ref()
            .unwrap()
            .simulate(self.playouts_per_simulation);
        self.back_propagate(node_key, result);
    }

    fn select(&self) -> MctsNodeKey {
        let tree = Rc::clone(&self.tree);
        let tree = tree.borrow();
        uct_select(&tree, self.exploration_constant)
    }

    fn expand(&mut self, node_key: MctsNodeKey) -> MctsNodeKey {
        let tree = Rc::clone(&self.tree);
        let mut tree = tree.borrow_mut();
        let node = tree.get_node_from_nodekey(node_key);
        if !node.children.is_empty() {
            panic!("unexpected expansion for node with children!");
        }
        let state = node.state.as_ref().unwrap();
        let actions = state.get_actions();

        // For each action, create a child with no state. We only create state during a simulation
        // that ends up choosing this child.
        for action in &actions {
            self.tree.borrow_mut().add_child(None, node_key, *action);
        }

        // Choose a random child, populate its state.
        let random_action = &actions.choose(&mut rand::thread_rng()).unwrap();
        let random_child = tree
            .get_children_nodekeys(node_key)
            .get(random_action)
            .unwrap();
        let new_state = state.get_next_state(random_action);

        let tree = Rc::clone(&self.tree);
        let mut tree = tree.borrow_mut();
        tree.get_mut_node_from_nodekey(*random_child).state = Some(new_state);

        *random_child
    }

    fn back_propagate(&mut self, node_key: MctsNodeKey, results: Vec<SimulationResult>) {
        let mut node_key = node_key;
        let tree = Rc::clone(&self.tree);
        let mut tree = tree.borrow_mut();
        for result in results {
            let node = tree.get_mut_node_from_nodekey(node_key);
            node.visits += 1;
            if result == SimulationResult::Win {
                node.wins += 1;
            }
            node_key = node.parent.unwrap();
        }
    }

    fn get_best_action(&self) -> Option<_Action> {
        let tree = self.tree.borrow();
        let root_nodekey = tree.get_root_nodekey();
        let mut best_action = None;
        let mut best_score = Int::MIN;
        for (action, child) in tree.get_children_nodekeys(root_nodekey) {
            let child_node = tree.get_node_from_nodekey(*child);
            if child_node.visits > best_score {
                best_score = child_node.visits;
                best_action = Some(*action);
            }
        }
        best_action
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum MyAction {
        Up,
        Down,
        Left,
        Right,
    }

    impl Action for MyAction {}

    // Some dummy state that is associated with MCTS nodes. You would put e.g. "whose turn is it",
    // "what is the board", etc. here. You need to state to know what applying the action does.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct MyState {
        pub data: u32,
    }

    // The State trait is a flag that says that DummyState implements the State trait and can be
    // used with MCTS.
    impl State<MyAction> for MyState {
        fn simulate(&self, playouts: Int) -> Vec<SimulationResult> {
            todo!()
        }

        fn get_actions(&self) -> Vec<MyAction> {
            todo!()
        }

        fn get_next_state(&self, action: &MyAction) -> Self {
            if *action == MyAction::Down {}
            todo!()
        }
    }

    type MyMcts = Mcts<MyState, MyAction>;
    type MyMctsTree = MctsTree<MyState, MyAction>;

    fn new_my_mcts() -> MyMcts {
        Mcts::new(
            MyState { data: 0 },
            IterationLimitKind::Iterations(1000),
            1.0,
            100,
        )
    }

    // Test a small pre-built tree from chapter 5 page 162
    // - Root node has 100 visits, 37 wins.
    //   - First child has 79 visits, 60 wins
    //     - First grandchild has 26 visits, 3 wins.
    //     - Second grandchild has 53 visits, 16 wins.
    //       - First great grandchild has 35 visits, 27 wins.
    //       - Second great grandchild has 18 visits, 10 wins.
    //   - Second child has 10 visits, 1 win.
    //     - First grandchild has 6 visits, 6 wins.
    //       - First great grandchild has 3 visits, 0 wins.
    //       - Second great grandchild has 3 visits, 0 wins.
    //     - Second grandchild has 4 visits, 3 wins.
    //   - Third child has 11 visits, 2 wins.
    fn build_test_tree() -> MyMctsTree {
        let root_state = MyState { data: 0 };
        let mut tree = MyMctsTree::new(root_state);
        let root_node = tree.get_mut_root();
        root_node.wins = 37;
        root_node.visits = 100;

        let first_child_nodekey = tree.add_child(
            Some(MyState { data: 0 }),
            tree.get_root_nodekey(),
            MyAction::Up,
        );
        let first_child = tree.get_mut_node_from_nodekey(first_child_nodekey);
        first_child.wins = 60;
        first_child.visits = 79;

        let first_grandchild_nodekey =
            tree.add_child(Some(MyState { data: 0 }), first_child_nodekey, MyAction::Up);
        let first_grandchild = tree.get_mut_node_from_nodekey(first_grandchild_nodekey);
        first_grandchild.wins = 3;
        first_grandchild.visits = 26;

        let second_grandchild_nodekey = tree.add_child(
            Some(MyState { data: 0 }),
            first_child_nodekey,
            MyAction::Right,
        );
        let second_grandchild = tree.get_mut_node_from_nodekey(second_grandchild_nodekey);
        second_grandchild.wins = 16;
        second_grandchild.visits = 53;

        let first_great_grandchild_nodekey = tree.add_child(
            Some(MyState { data: 0 }),
            second_grandchild_nodekey,
            MyAction::Up,
        );
        let first_great_grandchild = tree.get_mut_node_from_nodekey(first_great_grandchild_nodekey);
        first_great_grandchild.wins = 27;
        first_great_grandchild.visits = 35;

        let second_great_grandchild_nodekey = tree.add_child(
            Some(MyState { data: 0 }),
            second_grandchild_nodekey,
            MyAction::Right,
        );
        let second_great_grandchild =
            tree.get_mut_node_from_nodekey(second_great_grandchild_nodekey);
        second_great_grandchild.wins = 10;
        second_great_grandchild.visits = 18;

        let second_child_nodekey = tree.add_child(
            Some(MyState { data: 0 }),
            tree.get_root_nodekey(),
            MyAction::Right,
        );
        let second_child = tree.get_mut_node_from_nodekey(second_child_nodekey);
        second_child.wins = 1;
        second_child.visits = 10;

        let first_grandchild_nodekey = tree.add_child(
            Some(MyState { data: 0 }),
            second_child_nodekey,
            MyAction::Up,
        );
        let first_grandchild = tree.get_mut_node_from_nodekey(first_grandchild_nodekey);
        first_grandchild.wins = 6;
        first_grandchild.visits = 6;

        let first_great_grandchild_nodekey = tree.add_child(
            Some(MyState { data: 0 }),
            first_grandchild_nodekey,
            MyAction::Right,
        );
        let first_great_grandchild = tree.get_mut_node_from_nodekey(first_great_grandchild_nodekey);
        first_great_grandchild.wins = 0;
        first_great_grandchild.visits = 3;

        let second_great_grandchild_nodekey = tree.add_child(
            Some(MyState { data: 0 }),
            first_grandchild_nodekey,
            MyAction::Right,
        );
        let second_great_grandchild =
            tree.get_mut_node_from_nodekey(second_great_grandchild_nodekey);
        second_great_grandchild.wins = 0;
        second_great_grandchild.visits = 3;

        let second_grandchild_nodekey = tree.add_child(
            Some(MyState { data: 0 }),
            second_child_nodekey,
            MyAction::Right,
        );
        let second_grandchild = tree.get_mut_node_from_nodekey(second_grandchild_nodekey);
        second_grandchild.wins = 3;
        second_grandchild.visits = 4;

        let third_child_nodekey = tree.add_child(
            Some(MyState { data: 0 }),
            tree.get_root_nodekey(),
            MyAction::Down,
        );
        let third_child = tree.get_mut_node_from_nodekey(third_child_nodekey);
        third_child.wins = 2;
        third_child.visits = 11;

        tree
    }

    #[test]
    fn test_mcts_tree_root_starts_off_as_zero() {
        let root_state = MyState { data: 0 };
        let tree = MyMctsTree::new(root_state);
        let root_node = tree.get_root();
        assert_eq!(root_node.visits, 0);
        assert_eq!(root_node.wins, 0);
        assert!(root_node.children.is_empty());
    }

    #[test]
    fn test_uct_score_first_child() {
        let score = uct_score(79, 60, 100, 1.4);
        assert_abs_diff_eq!(score, 1.098, epsilon = 0.001);
    }

    #[test]
    fn test_uct_score_second_child() {
        let score = uct_score(10, 1, 100, 1.4);
        assert_abs_diff_eq!(score, 1.050, epsilon = 0.001);
    }

    #[test]
    fn test_uct_score_third_child() {
        let score = uct_score(11, 2, 100, 1.4);
        assert_abs_diff_eq!(score, 1.088, epsilon = 0.001);
    }

    // Test a small pre-built tree from chapter 5 page 162.
    //
    // As per p163, if C = 1.4, then the first child is selected. Unlike the book example,
    // rather than stop at the 60/79 node, we keep going all the way to the correct leaf node, which
    // is the 27/35 noe.
    #[test]
    fn test_mcts_tree_small_tree_c_14_first_child_selected() {
        let tree = build_test_tree();
        let selected_child_node_key = uct_select(&tree, 1.4);
        let selected_child = tree.get_node_from_nodekey(selected_child_node_key);
        assert_eq!(selected_child.visits, 35);
        assert_eq!(selected_child.wins, 27);
    }

    //
    // // Test a small pre-built tree from chapter 5 page 162, just first level.
    // //
    // // As per p163, if C = 1.5, then the third child is selected.
    #[test]
    fn test_mcts_tree_small_tree_c_15_third_child_selected() {
        let tree = build_test_tree();
        let selected_child_node_key = uct_select(&tree, 1.5);
        let selected_child = tree.get_node_from_nodekey(selected_child_node_key);
        assert_eq!(selected_child.visits, 11);
        assert_eq!(selected_child.wins, 2);
    }
}