use num_traits::real::Real;
use rand::seq::IteratorRandom;
use slotmap::new_key_type;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

trait Action: Clone + Copy + PartialEq + Eq + Hash {}
trait Float: Clone + Copy + Real {}
trait Int:
    Clone + Copy + num_traits::Num + num_traits::NumCast + num_traits::Bounded + Ord + PartialOrd
{
}
trait State {}

impl Action for u32 {}

// Using a macro, impl Float for all primitive float types
macro_rules! impl_float_for_all_float_types {
    ($($t:ty)*) => ($(
        impl Float for $t {}
    )*)
}
impl_float_for_all_float_types! { f32 f64 }

// Using a macro, impl Int for all primitive int types
macro_rules! impl_int_for_all_int_types {
    ($($t:ty)*) => ($(
        impl Int for $t {}
    )*)
}
impl_int_for_all_int_types! { u8 u16 u32 u64 usize i8 i16 i32 i64 }

new_key_type! { struct MctsNodeKey; }

#[derive(Debug, Clone)]
struct MctsNode<_State, _Action, _Int>
where
    _State: State,
    _Action: Action,
    _Int: Int,
{
    parent: Option<MctsNodeKey>,
    children: HashMap<_Action, MctsNodeKey>,
    visits: _Int,
    wins: _Int,
    state: Option<_State>,
}

impl<_State, _Action, _Int> MctsNode<_State, _Action, _Int>
where
    _State: State,
    _Action: Action,
    _Int: Int,
{
    fn new(parent: Option<MctsNodeKey>, state: Option<_State>) -> Self {
        Self {
            parent,
            children: HashMap::new(),
            visits: num_traits::zero(),
            wins: num_traits::zero(),
            state,
        }
    }
}

#[derive(Debug, Clone)]
struct MctsTree<_State, _Action, _Int>
where
    _State: State,
    _Action: Action,
    _Int: Int,
{
    nodes: slotmap::SlotMap<MctsNodeKey, MctsNode<_State, _Action, _Int>>,
    root: MctsNodeKey,
}

impl<_State, _Action, _Int> MctsTree<_State, _Action, _Int>
where
    _State: State,
    _Action: Action,
    _Int: Int,
{
    fn new(root_state: _State) -> Self {
        let mut nodes = slotmap::SlotMap::with_key();
        let root = nodes.insert(MctsNode::new(None, Some(root_state)));
        Self { nodes, root }
    }

    fn get_node_from_nodekey(&self, node: MctsNodeKey) -> &MctsNode<_State, _Action, _Int> {
        &self.nodes[node]
    }

    fn get_mut_node_from_nodekey(
        &mut self,
        node: MctsNodeKey,
    ) -> &mut MctsNode<_State, _Action, _Int> {
        &mut self.nodes[node]
    }

    fn get_root(&self) -> &MctsNode<_State, _Action, _Int> {
        &self.nodes[self.root]
    }

    fn get_mut_root(&mut self) -> &mut MctsNode<_State, _Action, _Int> {
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

fn uct_score<_Int, _Float>(
    node_visits: _Int,
    node_wins: _Int,
    parent_visits: _Int,
    exploration_constant: _Float,
) -> _Float
where
    _Int: Int,
    _Float: Float,
{
    let node_wins_float = num_traits::cast::<_, _Float>(node_wins).unwrap();
    let node_visits_float = num_traits::cast::<_, _Float>(node_visits).unwrap();
    let parent_visits_float = num_traits::cast::<_, _Float>(parent_visits).unwrap();
    let exploitation_term = node_wins_float / node_visits_float;
    let exploration_term =
        exploration_constant * (parent_visits_float.ln() / node_visits_float).sqrt();
    exploitation_term + exploration_term
}

enum SimulationResult {
    Win,
    NotWin,
}

trait StateActionApplier<_State, _Action>
where
    _State: State,
    _Action: Action,
{
    fn apply_action(&self, state: &_State, action: &_Action) -> _State;
}

trait Select<_State, _Action, _Int>
where
    _State: State,
    _Action: Action,
    _Int: Int,
{
    fn select(&self, tree: &MctsTree<_State, _Action, _Int>) -> MctsNodeKey;
}

trait Expand<_State, _Action, _Int>
where
    _State: State,
    _Action: Action,
    _Int: Int,
{
    fn expand(&self, tree: &mut MctsTree<_State, _Action, _Int>, node: MctsNodeKey) -> MctsNodeKey;
}

trait Simulate<_State>
where
    _State: State,
{
    fn simulate(&self, state: &_State) -> SimulationResult;
}

trait BackPropagate<_State, _Action, _Int>
where
    _State: State,
    _Action: Action,
    _Int: Int,
{
    fn back_propagate(
        &self,
        tree: &mut MctsTree<_State, _Action, _Int>,
        node: MctsNodeKey,
        result: SimulationResult,
    );
}

struct UctSelect<_State, _Action, _Int, _Float>
where
    _State: State,
    _Action: Action,
    _Int: Int,
    _Float: Float,
{
    exploration_constant: _Float,
    phantom_state: PhantomData<_State>,
    phantom_action: PhantomData<_Action>,
    phantom_int: PhantomData<_Int>,
}

impl<_State, _Action, _Int, _Float> Default for UctSelect<_State, _Action, _Int, _Float>
where
    _State: State,
    _Action: Action,
    _Int: Int,
    _Float: Float,
{
    fn default() -> Self {
        UctSelect::new(_Float::from(1.4).unwrap())
    }
}

impl<_State, _Action, _Int, _Float> UctSelect<_State, _Action, _Int, _Float>
where
    _State: State,
    _Action: Action,
    _Int: Int,
    _Float: Float,
{
    fn new(exploration_constant: _Float) -> Self {
        Self {
            exploration_constant,
            phantom_state: PhantomData,
            phantom_action: PhantomData,
            phantom_int: PhantomData,
        }
    }
}

impl<_State, _Action, _Int, _Float> Select<_State, _Action, _Int>
    for UctSelect<_State, _Action, _Int, _Float>
where
    _State: State,
    _Action: Action,
    _Int: Int,
    _Float: Float,
{
    fn select(&self, tree: &MctsTree<_State, _Action, _Int>) -> MctsNodeKey {
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
                        self.exploration_constant,
                    );
                    (action, child, score)
                })
                .max_by(|(_, _, score1), (_, _, score2)| score1.partial_cmp(score2).unwrap())
                .unwrap();
            node = *children.get(action).unwrap();
        }
        node
    }
}

struct ExpandAllSelectRandom<_State, _Action, _Int>
where
    _State: State,
    _Action: Action,
    _Int: Int,
{
    phantom_state: PhantomData<_State>,
    phantom_action: PhantomData<_Action>,
    phantom_int: PhantomData<_Int>,
}

impl<_State, _Action, _Int> Default for ExpandAllSelectRandom<_State, _Action, _Int>
where
    _State: State,
    _Action: Action,
    _Int: Int,
{
    fn default() -> Self {
        ExpandAllSelectRandom::new()
    }
}

impl<_State, _Action, _Int> ExpandAllSelectRandom<_State, _Action, _Int>
where
    _State: State,
    _Action: Action,
    _Int: Int,
{
    fn new() -> Self {
        Self {
            phantom_state: PhantomData,
            phantom_action: PhantomData,
            phantom_int: PhantomData,
        }
    }
}

impl<_State, _Action, _Int> Expand<_State, _Action, _Int>
    for ExpandAllSelectRandom<_State, _Action, _Int>
where
    _State: State,
    _Action: Action,
    _Int: Int,
{
    fn expand(&self, tree: &mut MctsTree<_State, _Action, _Int>, node: MctsNodeKey) -> MctsNodeKey {
        let mut node = node;
        loop {
            let children = tree.get_children_nodekeys(node);
            if children.is_empty() {
                break;
            }
            let mut rng = rand::thread_rng();
            let (action, _random_child) = children
                .iter()
                .choose(&mut rng)
                .expect("ExpandAllSelectRandom: children is empty");
            node = *children.get(action).unwrap();
        }
        node
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IterationLimitKind {
    Iterations(usize),
    TimeSeconds(Duration),
}

// Mcts is the main Monte Carlo Tree Search algorithm.
// See section 5.4 Monte Carlo Tree Search page 162 and 163.
struct Mcts<_State, _Action, _Int, _Select, _Expand, _Simulate, _BackPropagate, _StateActionApplier>
where
    _State: State,
    _Action: Action,
    _Int: Int,
    _Select: Select<_State, _Action, _Int>,
    _Expand: Expand<_State, _Action, _Int>,
    _Simulate: Simulate<_State>,
    _BackPropagate: BackPropagate<_State, _Action, _Int>,
    _StateActionApplier: StateActionApplier<_State, _Action>,
{
    select: _Select,
    expand: _Expand,
    simulate: _Simulate,
    back_propagate: _BackPropagate,
    state_action_applier: _StateActionApplier,
    tree: MctsTree<_State, _Action, _Int>,
    iteration_limit: IterationLimitKind,
}

impl<_State, _Action, _Int, _Select, _Expand, _Simulate, _BackPropagate, _StateActionApplier>
    Mcts<_State, _Action, _Int, _Select, _Expand, _Simulate, _BackPropagate, _StateActionApplier>
where
    _State: State,
    _Action: Action,
    _Int: Int,
    _Select: Select<_State, _Action, _Int>,
    _Expand: Expand<_State, _Action, _Int>,
    _Simulate: Simulate<_State>,
    _BackPropagate: BackPropagate<_State, _Action, _Int>,
    _StateActionApplier: StateActionApplier<_State, _Action>,
{
    fn new(
        select: _Select,
        expand: _Expand,
        simulate: _Simulate,
        back_propagate: _BackPropagate,
        state_action_applier: _StateActionApplier,
        root_state: _State,
        iteration_limit: IterationLimitKind,
    ) -> Self {
        Self {
            select,
            expand,
            simulate,
            back_propagate,
            state_action_applier,
            tree: MctsTree::new(root_state),
            iteration_limit,
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
        let node_key = self.select.select(&self.tree);
        let node_key = self.expand.expand(&mut self.tree, node_key);
        let node = self.tree.get_node_from_nodekey(node_key);
        let result = self.simulate.simulate(node.state.as_ref().unwrap());
        self.back_propagate
            .back_propagate(&mut self.tree, node_key, result);
    }

    fn get_best_action(&self) -> Option<_Action> {
        let root_nodekey = self.tree.get_root_nodekey();
        let mut best_action = None;
        let mut best_score = _Int::min_value();
        for (action, child) in self.tree.get_children_nodekeys(root_nodekey) {
            let child_node = self.tree.get_node_from_nodekey(*child);
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
    use super::*;
    use approx::assert_abs_diff_eq;

    // Some dummy state that is associated with MCTS nodes. You would put e.g. "whose turn is it",
    // "what is the board", etc. here. You need to state to know what applying the action does.
    struct DummyState {}

    // The State trait is a flag that says that DummyState implements the State trait and can be
    // used with MCTS.
    impl State for DummyState {}

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
    fn build_test_tree() -> MctsTree<DummyState, u32, u32> {
        let mut tree = MctsTree::<DummyState, u32, u32>::new(DummyState {});
        let root_node = tree.get_mut_root();
        root_node.wins = 37;
        root_node.visits = 100;

        let first_child_nodekey = tree.add_child(Some(DummyState {}), tree.root, 1);
        let first_child = tree.get_mut_node_from_nodekey(first_child_nodekey);
        first_child.wins = 60;
        first_child.visits = 79;

        let first_grandchild_nodekey = tree.add_child(Some(DummyState {}), first_child_nodekey, 1);
        let first_grandchild = tree.get_mut_node_from_nodekey(first_grandchild_nodekey);
        first_grandchild.wins = 3;
        first_grandchild.visits = 26;

        let second_grandchild_nodekey = tree.add_child(Some(DummyState {}), first_child_nodekey, 2);
        let second_grandchild = tree.get_mut_node_from_nodekey(second_grandchild_nodekey);
        second_grandchild.wins = 16;
        second_grandchild.visits = 53;

        let first_great_grandchild_nodekey =
            tree.add_child(Some(DummyState {}), second_grandchild_nodekey, 1);
        let first_great_grandchild = tree.get_mut_node_from_nodekey(first_great_grandchild_nodekey);
        first_great_grandchild.wins = 27;
        first_great_grandchild.visits = 35;

        let second_great_grandchild_nodekey =
            tree.add_child(Some(DummyState {}), second_grandchild_nodekey, 2);
        let second_great_grandchild =
            tree.get_mut_node_from_nodekey(second_great_grandchild_nodekey);
        second_great_grandchild.wins = 10;
        second_great_grandchild.visits = 18;

        let second_child_nodekey = tree.add_child(Some(DummyState {}), tree.root, 2);
        let second_child = tree.get_mut_node_from_nodekey(second_child_nodekey);
        second_child.wins = 1;
        second_child.visits = 10;

        let first_grandchild_nodekey = tree.add_child(Some(DummyState {}), second_child_nodekey, 1);
        let first_grandchild = tree.get_mut_node_from_nodekey(first_grandchild_nodekey);
        first_grandchild.wins = 6;
        first_grandchild.visits = 6;

        let first_great_grandchild_nodekey =
            tree.add_child(Some(DummyState {}), first_grandchild_nodekey, 1);
        let first_great_grandchild = tree.get_mut_node_from_nodekey(first_great_grandchild_nodekey);
        first_great_grandchild.wins = 0;
        first_great_grandchild.visits = 3;

        let second_great_grandchild_nodekey =
            tree.add_child(Some(DummyState {}), first_grandchild_nodekey, 2);
        let second_great_grandchild =
            tree.get_mut_node_from_nodekey(second_great_grandchild_nodekey);
        second_great_grandchild.wins = 0;
        second_great_grandchild.visits = 3;

        let second_grandchild_nodekey =
            tree.add_child(Some(DummyState {}), second_child_nodekey, 2);
        let second_grandchild = tree.get_mut_node_from_nodekey(second_grandchild_nodekey);
        second_grandchild.wins = 3;
        second_grandchild.visits = 4;

        let third_child_nodekey = tree.add_child(Some(DummyState {}), tree.root, 3);
        let third_child = tree.get_mut_node_from_nodekey(third_child_nodekey);
        third_child.wins = 2;
        third_child.visits = 11;

        tree
    }

    #[test]
    fn test_mcts_tree_root_starts_off_as_zero() {
        let tree = MctsTree::<DummyState, u32, u32>::new(DummyState {});
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
    // As per p163, if C = 1.4, then the first child is selected.
    // #[test]
    // fn test_mcts_tree_small_tree_c_14_first_child_selected() {
    //     let tree = build_test_tree();
    //
    //     type MctsImpl = Mcts<
    //         u32 /*_State*/,
    //         u32 /*_Action*/,
    //         u32 /*_Int*/,
    //         UctSelect<u32, u32, >
    //
    //     let mcts = Mcts::<u32, f32, u32, DummyState>::new(1.4);
    //
    //     let selection_policy = UctSelectionPolicy::<u32, f32, u32, DummyState>::new(1.4);
    //     let selected_child = selection_policy.select_child(&tree, tree.root);
    //     assert!(selected_child.is_some());
    //     let selected_child = selected_child.unwrap();
    //     let selected_child = tree.get_node_from_nodekey(selected_child);
    //     assert_eq!(selected_child.visits, 79);
    //     assert_eq!(selected_child.wins, 60);
    // }

    //
    // // Test a small pre-built tree from chapter 5 page 162, just first level.
    // //
    // // As per p163, if C = 1.5, then the third child is selected.
    // #[test]
    // fn test_mcts_tree_small_tree_c_15_third_child_selected() {
    //     let tree = build_test_tree();
    //     let selection_policy = UctSelectionPolicy::<u32, f32, u32, DummyState>::new(1.5);
    //     let selected_child = selection_policy.select_child(&tree, tree.root);
    //     assert!(selected_child.is_some());
    //     let selected_child = selected_child.unwrap();
    //     let selected_child = tree.get_node_from_nodekey(selected_child);
    //     assert_eq!(selected_child.visits, 11);
    //     assert_eq!(selected_child.wins, 2);
    // }
}
