use num_traits::real::Real;
use slotmap::new_key_type;
use std::collections::HashMap;
use std::hash::Hash;

trait Action: Clone + Copy + PartialEq + Eq + Hash {}
trait Float: Clone + Copy + num_traits::Float {}
trait Int: Clone + Copy + num_traits::Num + num_traits::NumCast + num_traits::FromPrimitive {}
trait State {}

impl Action for u32 {}
impl Float for f32 {}
impl Float for f64 {}
impl Int for u32 {}

new_key_type! { struct MctsNodeKey; }

#[derive(Debug, Clone)]
struct MctsNode<_Action, _Int, _State>
where
    _Action: Action,
    _Int: Int,
    _State: State,
{
    parent: Option<MctsNodeKey>,
    children: HashMap<_Action, MctsNodeKey>,
    visits: _Int,
    wins: _Int,
    state: _State,
}

impl<_Action, _Int, _State> MctsNode<_Action, _Int, _State>
where
    _Action: Action,
    _Int: Int,
    _State: State,
{
    fn new(parent: Option<MctsNodeKey>, state: _State) -> Self {
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
struct MctsTree<_Action, _Int, _State>
where
    _Action: Action,
    _Int: Int,
    _State: State,
{
    nodes: slotmap::SlotMap<MctsNodeKey, MctsNode<_Action, _Int, _State>>,
    root: MctsNodeKey,
}

impl<_Action, _Int, _State> MctsTree<_Action, _Int, _State>
where
    _Action: Action,
    _Int: Int,
    _State: State,
{
    fn new(root_state: _State) -> Self {
        let mut nodes = slotmap::SlotMap::with_key();
        let root = nodes.insert(MctsNode::new(None, root_state));
        Self { nodes, root }
    }

    fn get_node_from_nodekey(&self, node: MctsNodeKey) -> &MctsNode<_Action, _Int, _State> {
        &self.nodes[node]
    }

    fn get_mut_node_from_nodekey(
        &mut self,
        node: MctsNodeKey,
    ) -> &mut MctsNode<_Action, _Int, _State> {
        &mut self.nodes[node]
    }

    fn get_root(&self) -> &MctsNode<_Action, _Int, _State> {
        &self.nodes[self.root]
    }

    fn get_mut_root(&mut self) -> &mut MctsNode<_Action, _Int, _State> {
        &mut self.nodes[self.root]
    }

    fn get_root_nodekey(&self) -> MctsNodeKey {
        self.root
    }

    fn get_children_nodekeys(&self, node: MctsNodeKey) -> &HashMap<_Action, MctsNodeKey> {
        &self.nodes[node].children
    }

    fn add_child(&mut self, state: _State, parent: MctsNodeKey, action: _Action) -> MctsNodeKey {
        let child = self.nodes.insert(MctsNode::new(Some(parent), state));
        self.nodes[parent].children.insert(action, child);
        child
    }

    fn add_mut_child(
        &mut self,
        state: _State,
        parent: MctsNodeKey,
        action: _Action,
    ) -> &mut MctsNode<_Action, _Int, _State> {
        let child = self.nodes.insert(MctsNode::new(Some(parent), state));
        self.nodes[parent].children.insert(action, child);
        &mut self.nodes[child]
    }
}

// SelectionPolicy is a trait that defines how to select a child node from a parent node.
// Implementations need to balance exploration vs exploitation. See section 5.4 Monte Carlo Tree
// Search page 162 and 163.
trait SelectionPolicy {
    type _Action: Action;
    type _Int: Int;
    type _State: State;

    fn select_child(
        &self,
        tree: &MctsTree<Self::_Action, Self::_Int, Self::_State>,
        node: MctsNodeKey,
    ) -> Option<MctsNodeKey>;
}

// UctSelectionPolicy implements the UCT selection policy.
// See section 5.4 Monte Carlo Tree Search page 162 and 163.
struct UctSelectionPolicy<_Action, _Float, _Int, _State>
where
    _Action: Action,
    _Float: Float,
    _Int: Int,
    _State: State,
{
    exploration_constant: _Float,
    phantom_action: std::marker::PhantomData<_Action>,
    phantom_int: std::marker::PhantomData<_Int>,
    phantom_state: std::marker::PhantomData<_State>,
}

impl<_Action, _Float, _Int, _State> UctSelectionPolicy<_Action, _Float, _Int, _State>
where
    _Action: Action,
    _Float: Float,
    _Int: Int,
    _State: State,
{
    fn new(exploration_constant: _Float) -> Self {
        Self {
            exploration_constant,
            phantom_action: std::marker::PhantomData,
            phantom_int: std::marker::PhantomData,
            phantom_state: std::marker::PhantomData,
        }
    }

    fn uct_score(&self, node_visits: _Int, node_wins: _Int, parent_visits: _Int) -> _Float {
        let node_wins_float = num_traits::cast::<_, _Float>(node_wins).unwrap();
        let node_visits_float = num_traits::cast::<_, _Float>(node_visits).unwrap();
        let parent_visits_float = num_traits::cast::<_, _Float>(parent_visits).unwrap();
        let exploitation_term = node_wins_float / node_visits_float;
        let exploration_term =
            self.exploration_constant * (parent_visits_float.ln() / node_visits_float).sqrt();
        exploitation_term + exploration_term
    }
}

impl<_Action, _Float, _Int, _State> SelectionPolicy
    for UctSelectionPolicy<_Action, _Float, _Int, _State>
where
    _Action: Action,
    _Float: Float,
    _Int: Int,
    _State: State,
{
    type _Action = _Action;
    type _Int = _Int;
    type _State = _State;

    fn select_child(
        &self,
        tree: &MctsTree<Self::_Action, Self::_Int, Self::_State>,
        node: MctsNodeKey,
    ) -> Option<MctsNodeKey> {
        let parent_visits = tree.get_node_from_nodekey(node).visits;
        let mut best_child = None;
        let mut best_score = _Float::neg_infinity();
        for child in tree.get_children_nodekeys(node).values() {
            let child_node = tree.get_node_from_nodekey(*child);
            let score = self.uct_score(child_node.visits, child_node.wins, parent_visits);
            if score > best_score {
                best_score = score;
                best_child = Some(*child);
            }
        }
        best_child
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

    // Test a small pre-built tree from chapter 5 page 162, just first level.
    // - Root node has 100 visits, 37 wins.
    // - Root node has 3 children.
    //   - First child has 79 visits, 19 wins
    //   - Second child has 10 visits, 9 wins.
    //   - Third child has 11 visits, 9 wins.
    fn build_test_tree() -> MctsTree<u32, u32, DummyState> {
        let mut tree = MctsTree::<u32, u32, DummyState>::new(DummyState {});
        let root_node = tree.get_mut_root();
        root_node.wins = 37;
        root_node.visits = 100;

        let first_child = tree.add_mut_child(DummyState {}, tree.root, 1);
        first_child.wins = 60;
        first_child.visits = 79;

        let second_child = tree.add_mut_child(DummyState {}, tree.root, 2);
        second_child.wins = 1;
        second_child.visits = 10;

        let third_child = tree.add_mut_child(DummyState {}, tree.root, 3);
        third_child.wins = 2;
        third_child.visits = 11;

        tree
    }

    #[test]
    fn test_mcts_tree_root_starts_off_as_zero() {
        let tree = MctsTree::<u32, u32, DummyState>::new(DummyState {});
        let root_node = tree.get_root();
        assert_eq!(root_node.visits, 0);
        assert_eq!(root_node.wins, 0);
        assert!(root_node.children.is_empty());
    }

    #[test]
    fn test_uct_score_first_child() {
        let selection_policy = UctSelectionPolicy::<u32, f32, u32, DummyState>::new(1.4);
        let score = selection_policy.uct_score(79, 60, 100);
        assert_abs_diff_eq!(score, 1.098, epsilon = 0.001);
    }

    #[test]
    fn test_uct_score_second_child() {
        let selection_policy = UctSelectionPolicy::<u32, f32, u32, DummyState>::new(1.4);
        let score = selection_policy.uct_score(10, 1, 100);
        assert_abs_diff_eq!(score, 1.050, epsilon = 0.001);
    }

    #[test]
    fn test_uct_score_third_child() {
        let selection_policy = UctSelectionPolicy::<u32, f32, u32, DummyState>::new(1.4);
        let score = selection_policy.uct_score(11, 2, 100);
        assert_abs_diff_eq!(score, 1.088, epsilon = 0.001);
    }

    // Test a small pre-built tree from chapter 5 page 162, just first level.
    //
    // As per p163, if C = 1.4, then the first child is selected.
    #[test]
    fn test_mcts_tree_small_tree_c_14_first_child_selected() {
        let tree = build_test_tree();
        let selection_policy = UctSelectionPolicy::<u32, f32, u32, DummyState>::new(1.4);
        let selected_child = selection_policy.select_child(&tree, tree.root);
        assert!(selected_child.is_some());
        let selected_child = selected_child.unwrap();
        let selected_child = tree.get_node_from_nodekey(selected_child);
        assert_eq!(selected_child.visits, 79);
        assert_eq!(selected_child.wins, 60);
    }

    // Test a small pre-built tree from chapter 5 page 162, just first level.
    //
    // As per p163, if C = 1.5, then the third child is selected.
    #[test]
    fn test_mcts_tree_small_tree_c_15_third_child_selected() {
        let tree = build_test_tree();
        let selection_policy = UctSelectionPolicy::<u32, f32, u32, DummyState>::new(1.5);
        let selected_child = selection_policy.select_child(&tree, tree.root);
        assert!(selected_child.is_some());
        let selected_child = selected_child.unwrap();
        let selected_child = tree.get_node_from_nodekey(selected_child);
        assert_eq!(selected_child.visits, 11);
        assert_eq!(selected_child.wins, 2);
    }
}
