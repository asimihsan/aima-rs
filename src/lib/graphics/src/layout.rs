/*
 * Copyright (C) 2023 Asim Ihsan
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU Affero General Public License as published by the Free
 * Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
 * PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License along
 * with this program. If not, see <https://www.gnu.org/licenses/>
 */

use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

#[derive(Debug, Clone, Copy)]
struct Position {
    x: f64,
    y: f64,
    modifier: f64,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            x: 0.0,
            y: 0.0,
            modifier: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Size {
    width: f64,
    height: f64,
}

pub trait Data: Debug + Clone {}

static NODE_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub type NodeWeakRef<_Data> = Weak<RefCell<Node<_Data>>>;
pub type NodeRef<_Data> = Rc<RefCell<Node<_Data>>>;

/// Node in a tree of layout objects. Each node has a parent (except the root) and children.
///
/// See: https://timidger.github.io/posts/designing-a-bi-mutable-directional-tree-safely-in-rust/
#[derive(Debug, Clone)]
struct Node<_Data>
where
    _Data: Data,
{
    id: usize,
    data: _Data,
    _parent: Option<NodeWeakRef<_Data>>,
    _children: Vec<NodeRef<_Data>>,
    position: Position,
    size: Size,
}

/// DebugNode is a kind of node that does not use references, it clones everything. This makes it
/// easier to debug and serialize for debugging. There are no parent pointers in DebugNode because
/// this allows us to avoid needing Box<_> in the children Vec.
#[derive(Debug, Clone)]
struct DebugNode<_Data>
where
    _Data: Data,
{
    id: usize,
    data: _Data,
    children: Vec<DebugNode<_Data>>,
    position: Position,
    size: Size,
}

impl<_Data: Data> From<Node<_Data>> for DebugNode<_Data> {
    fn from(node: Node<_Data>) -> Self {
        DebugNode {
            id: node.id,
            data: node.data,
            children: node
                ._children
                .into_iter()
                .map(|x| x.borrow().clone().into())
                .collect(),
            position: node.position,
            size: node.size,
        }
    }
}

/// add_child both adds the child to the parent's children and sets the parent of the child.
///
/// This cannot be a method on Node because it requires a fresh NodeRef<_Data> to
/// the parent so that we can downgrade it to a Weak<RefCell<Node<_Data>>>.
fn add_child<_Data: Data>(parent: NodeRef<_Data>, child: NodeRef<_Data>) {
    parent.borrow_mut().add_child(child.clone());
    child.borrow_mut().set_parent(parent);
}

impl<_Data> Node<_Data>
where
    _Data: Data,
{
    fn new(size: Size, data: _Data) -> Rc<RefCell<Self>> {
        // Ordering is relaxed because we don't care about the order of the ids, just that they are
        // unique.
        let id = NODE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Rc::new(RefCell::new(Node {
            id,
            data,
            _parent: None,
            _children: Vec::new(),
            position: Position::default(),
            size,
        }))
    }

    fn add_child(&mut self, child: NodeRef<_Data>) {
        self._children.push(child);
    }

    fn set_parent(&mut self, parent: NodeRef<_Data>) {
        self._parent = Some(Rc::downgrade(&parent));
    }

    fn is_leaf(&self) -> bool {
        self._children.is_empty()
    }

    fn parent(&self) -> Option<NodeRef<_Data>> {
        self._parent.as_ref().map(|x| x.upgrade().unwrap())
    }

    fn is_leftmost(&self) -> bool {
        match self.parent() {
            Some(parent) => parent.borrow().leftmost_child().unwrap().borrow().id == self.id,
            None => true,
        }
    }

    fn is_rightmost(&self) -> bool {
        match self.parent() {
            Some(parent) => parent.borrow().rightmost_child().unwrap().borrow().id == self.id,
            None => true,
        }
    }

    fn children(&self) -> Vec<NodeRef<_Data>> {
        self._children.to_vec()
    }

    fn leftmost_child(&self) -> Option<NodeRef<_Data>> {
        self._children.first().cloned()
    }

    fn rightmost_child(&self) -> Option<NodeRef<_Data>> {
        self._children.last().cloned()
    }

    fn is_leftmost_child(&self) -> bool {
        match self.leftmost_child() {
            Some(leftmost_child) => leftmost_child.borrow().id == self.id,
            None => false,
        }
    }

    fn is_rightmost_child(&self) -> bool {
        match self.rightmost_child() {
            Some(rightmost_child) => rightmost_child.borrow().id == self.id,
            None => false,
        }
    }

    fn previous_sibling(&self) -> Option<NodeRef<_Data>> {
        let parent = self.parent()?;
        let children = parent.borrow().children();
        let index = children
            .iter()
            .position(|x| x.borrow().id == self.id)
            .unwrap();
        if index == 0 {
            None
        } else {
            Some(children[index - 1].clone())
        }
    }

    fn next_sibling(&self) -> Option<NodeRef<_Data>> {
        let parent = self.parent()?;
        let children = parent.borrow().children();
        let index = children
            .iter()
            .position(|x| x.borrow().id == self.id)
            .unwrap();
        if index == children.len() - 1 {
            None
        } else {
            Some(children[index + 1].clone())
        }
    }

    fn leftmost_sibling(&self) -> Option<NodeRef<_Data>> {
        let parent = self.parent()?;
        let children = parent.borrow().children();
        Some(children.first().unwrap().clone())
    }

    fn rightmost_sibling(&self) -> Option<NodeRef<_Data>> {
        let parent = self.parent()?;
        let children = parent.borrow().children();
        Some(children.last().unwrap().clone())
    }
}

impl<_Data> PartialEq for Node<_Data>
where
    _Data: Data,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Tree of layout objects. The tree is immutable, but the layout objects themselves are
/// mutable.
#[derive(Debug, Clone)]
struct Tree<_Data>
where
    _Data: Data,
{
    _root: NodeRef<_Data>,
}

impl<_Data: Data> Tree<_Data> {
    fn new(root_data: _Data, root_size: Size) -> Self {
        let root = Node::new(root_size, root_data);
        Tree { _root: root }
    }

    fn root(&self) -> NodeRef<_Data> {
        self._root.clone()
    }
}

#[derive(Debug, Clone)]
struct DebugTree<_Data: Data> {
    root: DebugNode<_Data>,
}

impl<_Data: Data> From<Tree<_Data>> for DebugTree<_Data> {
    fn from(tree: Tree<_Data>) -> Self {
        let root: Node<_Data> = tree.root().borrow().clone();
        DebugTree {
            root: DebugNode::from(root),
        }
    }
}

trait Layout<_Data>
where
    _Data: Data,
{
    fn layout(&self, tree: &mut Tree<_Data>);
    fn sibling_separation(&self) -> f64;
    fn tree_distance(&self) -> f64;
    fn node_size(&self) -> i32;
}

struct ReingoldTilfordLayout<_Data>
where
    _Data: Data,
{
    sibling_separation: f64,
    tree_distance: f64,
    node_size: i32,
    phantom_data: std::marker::PhantomData<_Data>,
}

impl<_Data: Data> ReingoldTilfordLayout<_Data> {
    fn new(sibling_separation: f64, tree_distance: f64, node_size: i32) -> Self {
        ReingoldTilfordLayout {
            sibling_separation,
            tree_distance,
            node_size,
            phantom_data: std::marker::PhantomData,
        }
    }

    // initialize x to -1, y to depth, and mod to 0 for each node. depth
    // is the depth of the node in the tree. The root node is at depth 0.
    fn initialize_nodes(&self, node: NodeRef<_Data>, depth: f64) {
        let mut node = node.borrow_mut();
        node.position.x = -1.0;
        node.position.y = depth;
        node.position.modifier = 0.0;
        for child in node.children() {
            self.initialize_nodes(child, depth + 1.0);
        }
    }

    fn calculate_initial_x(&self, node: NodeRef<_Data>) {
        for child in node.borrow().children() {
            self.calculate_initial_x(child);
        }

        let node_position = node.borrow().position;

        // If no children
        if node.borrow().is_leaf() {
            // If this is the first node in a set, set its x to 0
            if node.borrow().is_leftmost() {
                node.borrow_mut().position.x = 0.0;
            // Otherwise, set its x to the x of its previous sibling plus the sibling separation
            } else {
                let previous_sibling = node.borrow().previous_sibling().unwrap();
                let previous_sibling = previous_sibling.borrow();
                node.borrow_mut().position.x =
                    previous_sibling.position.x + self.node_size as f64 + self.sibling_separation;
            }
        // If there is only one child
        } else if node.borrow().children().len() == 1 {
            let child = node.borrow().leftmost_child().unwrap();
            let child = child.borrow();

            // if this is the first node in a set, set its X value equal to its child's X value
            if node.borrow().is_leftmost() {
                node.borrow_mut().position.x = child.position.x;
            } else {
                // Otherwise, set its x to the x of its previous sibling plus the sibling separation
                let previous_sibling = node.borrow().previous_sibling().unwrap();
                let previous_sibling = previous_sibling.borrow();
                node.borrow_mut().position.x =
                    previous_sibling.position.x + self.node_size as f64 + self.sibling_separation;
                node.borrow_mut().position.modifier = node_position.x - child.position.x;
            }
        } else {
            let leftmost_child = node.borrow().leftmost_child().unwrap();
            let leftmost_child = leftmost_child.borrow();
            let rightmost_child = node.borrow().rightmost_child().unwrap();
            let rightmost_child = rightmost_child.borrow();
            let mid = (leftmost_child.position.x + rightmost_child.position.x) / 2.0;

            // if node is left most, set its x to the midpoint of its children
            if node.borrow().is_leftmost() {
                node.borrow_mut().position.x = mid;
            } else {
                // Otherwise, set its x to the x of its previous sibling plus the sibling separation
                let previous_sibling = node.borrow().previous_sibling().unwrap();
                let previous_sibling = previous_sibling.borrow();
                node.borrow_mut().position.x =
                    previous_sibling.position.x + self.node_size as f64 + self.sibling_separation;
                node.borrow_mut().position.modifier = node_position.x - mid;
            }
        }
    }

    fn check_for_conflicts(&self, _tree: &mut Tree<_Data>) {}
}

impl<_Data: Data> Layout<_Data> for ReingoldTilfordLayout<_Data> {
    fn layout(&self, tree: &mut Tree<_Data>) {
        self.initialize_nodes(tree.root(), 0.0 /*depth*/);
        self.calculate_initial_x(tree.root());
    }

    fn sibling_separation(&self) -> f64 {
        self.sibling_separation
    }

    fn tree_distance(&self) -> f64 {
        self.tree_distance
    }

    fn node_size(&self) -> i32 {
        self.node_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::VecDeque;

    #[derive(Debug, Clone)]
    struct TestNodeData {
        name: String,
    }

    impl Data for TestNodeData {}

    fn create_test_tree() -> Tree<TestNodeData> {
        let tree = Tree::new(
            TestNodeData {
                name: "root".to_string(),
            },
            Size {
                width: 100.0,
                height: 100.0,
            },
        );
        let root = tree.root();
        let child1 = Node::new(
            Size {
                width: 100.0,
                height: 100.0,
            },
            TestNodeData {
                name: "child1".to_string(),
            },
        );
        add_child(root.clone(), child1);

        let child2 = Node::new(
            Size {
                width: 100.0,
                height: 100.0,
            },
            TestNodeData {
                name: "child2".to_string(),
            },
        );
        add_child(root.clone(), child2);

        let child3 = Node::new(
            Size {
                width: 100.0,
                height: 100.0,
            },
            TestNodeData {
                name: "child3".to_string(),
            },
        );
        add_child(root.clone(), child3.clone());

        let child3_1 = Node::new(
            Size {
                width: 100.0,
                height: 100.0,
            },
            TestNodeData {
                name: "child3_1".to_string(),
            },
        );
        add_child(child3.clone(), child3_1);

        let child3_2 = Node::new(
            Size {
                width: 100.0,
                height: 100.0,
            },
            TestNodeData {
                name: "child3_2".to_string(),
            },
        );
        add_child(child3.clone(), child3_2);

        let child3_3 = Node::new(
            Size {
                width: 100.0,
                height: 100.0,
            },
            TestNodeData {
                name: "child3_3".to_string(),
            },
        );
        add_child(child3.clone(), child3_3);

        tree
    }

    // test you can create a tree with a root node and three children.
    #[test]
    fn test_create_tree() {
        let tree = create_test_tree();
        let root = tree.root();
        let root = root.borrow();
        assert_eq!(root.data.name, "root");
        assert_eq!(root.children().len(), 3);

        // for each child, check that the parent is the root
        for child in root.children() {
            let child = child.borrow();
            assert_eq!(child.parent().unwrap().borrow().id, root.id);
        }
    }

    // test that in the test tree, the first two children are leaves, the third child is not, and the root is not.
    #[test]
    fn test_is_leaf() {
        let tree = create_test_tree();
        let root = tree.root();
        let root = root.borrow();
        assert!(!root.is_leaf());

        let children = root.children();
        assert!(children[0].borrow().is_leaf());
        assert!(children[1].borrow().is_leaf());
        assert!(!children[2].borrow().is_leaf());
    }

    // test that only the first child of the root is the leftmost child.
    #[test]
    fn test_is_leftmost_child() {
        let tree = create_test_tree();
        let root = tree.root();
        let root = root.borrow();
        let children = root.children();

        assert_eq!(
            children[0].borrow().id,
            root.leftmost_child().unwrap().borrow().id
        );
    }

    // test that only the last child of the root is the rightmost child.
    #[test]
    fn test_is_rightmost_child() {
        let tree = create_test_tree();
        let root = tree.root();
        let root = root.borrow();
        let children = root.children();

        assert_eq!(
            children[2].borrow().id,
            root.rightmost_child().unwrap().borrow().id
        );
    }

    // test that get_previous_sibling returns the correct sibling.
    #[test]
    fn test_get_previous_sibling() {
        let tree = create_test_tree();
        let root = tree.root();
        let root = root.borrow();

        assert_eq!(root.children()[0].borrow().previous_sibling(), None);
        assert_eq!(
            root.children()[1].borrow().previous_sibling(),
            Some(root.children()[0].clone())
        );
        assert_eq!(
            root.children()[2].borrow().previous_sibling(),
            Some(root.children()[1].clone())
        )
    }

    // test that get_next_sibling returns the correct sibling.
    #[test]
    fn test_get_next_sibling() {
        let tree = create_test_tree();
        let root = tree.root();
        let root = root.borrow();
        let children = root.children();

        assert_eq!(
            children[0].borrow().next_sibling(),
            Some(children[1].clone())
        );
        assert_eq!(
            children[1].borrow().next_sibling(),
            Some(children[2].clone())
        );
        assert_eq!(children[2].borrow().next_sibling(), None);
    }

    // test get_leftmost_sibling returns the leftmost sibling.
    #[test]
    fn test_get_leftmost_sibling() {
        let tree = create_test_tree();
        let root = tree.root();
        let root = root.borrow();
        let children = root.children();

        assert_eq!(
            children[0].borrow().leftmost_sibling(),
            Some(children[0].clone())
        );
        assert_eq!(
            children[1].borrow().leftmost_sibling(),
            Some(children[0].clone())
        );
        assert_eq!(
            children[2].borrow().leftmost_sibling(),
            Some(children[0].clone())
        );
    }

    // test get_leftmost_child returns the leftmost child.
    #[test]
    fn test_get_leftmost_child() {
        let tree = create_test_tree();
        let root = tree.root();
        let root = root.borrow();
        let children = root.children();

        assert_eq!(root.leftmost_child(), Some(children[0].clone()));
    }

    // test get_rightmost_child returns the rightmost child.
    #[test]
    fn test_get_rightmost_child() {
        let tree = create_test_tree();
        let root = tree.root();
        let root = root.borrow();
        let children = root.children();

        assert_eq!(root.rightmost_child(), Some(children[2].clone()));
    }

    // test initialize_nodes initializes the nodes correctly. For all nodex X == -1, mod == 0,
    // and the depth is correct.
    #[test]
    fn test_reingold_tilford_initialize_nodes() {
        let mut tree = create_test_tree();
        let node_size = 1;
        let sibling_distance = 1.0;
        let tree_distance = 2.0;
        let layout = ReingoldTilfordLayout::new(sibling_distance, tree_distance, node_size);
        layout.initialize_nodes(tree.root(), 0.0 /*depth*/);

        let root = tree.root();
        let root = root.borrow();
        assert_eq!(root.position.x, -1.0);
        assert_eq!(root.position.modifier, 0.0);
        assert_eq!(root.position.y, 0.0);
        for child in root.children() {
            assert_eq!(child.borrow().position.x, -1.0);
            assert_eq!(child.borrow().position.modifier, 0.0);
            assert_eq!(child.borrow().position.y, 1.0);
        }
    }

    fn arbitrary_tree(max_iterations: usize) -> impl Strategy<Value = Tree<TestNodeData>> {
        let tree = Tree::new(
            TestNodeData {
                name: "root".to_string(),
            },
            Size {
                width: 100.0,
                height: 100.0,
            },
        );
        let node = tree.root();
        let node = node.borrow();

        Just(tree)
    }

    /// arbitrary_node creates a node adds it to the parent.
    fn arbitrary_node(
        parent: Rc<RefCell<Node<TestNodeData>>>,
    ) -> impl Strategy<Value = Rc<RefCell<Node<TestNodeData>>>> {
        let node = Node::new(
            Size {
                width: 100.0,
                height: 100.0,
            },
            TestNodeData {
                name: "node".to_string(),
            },
        );
        parent.borrow_mut().add_child(node.clone());
        Just(node)
    }

    #[test]
    fn test_reingold_tilford_calculate_x() {
        let tree = create_test_tree();
        let sibling_distance = 1.0;
        let tree_distance = 2.0;
        let node_size = 1;
        let layout = ReingoldTilfordLayout::new(sibling_distance, tree_distance, node_size);
        layout.initialize_nodes(tree.root(), 0.0 /*depth*/);
        layout.calculate_initial_x(tree.root());

        let _debug_tree: DebugTree<TestNodeData> = tree.into();
        let mut queue = VecDeque::new();
        queue.push_back(_debug_tree.root);
        while let Some(node) = queue.pop_front() {
            println!("{:?} {:?}", node.data.name, node.position);
            for child in node.children {
                queue.push_back(child);
            }
        }
    }

    proptest! {}
}
