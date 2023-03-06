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

use std::fmt::Debug;

use slotmap::new_key_type;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
struct Size {
    width: f64,
    height: f64,
}

pub trait Data: Debug + Clone {}

new_key_type! { struct NodeKey; }

/// Node in a tree of layout objects. Each node has a parent (except the root) and children.
#[derive(Debug, Clone)]
struct Node<_Data>
where
    _Data: Data,
{
    data: _Data,
    parent: Option<NodeKey>,
    children: Vec<NodeKey>,
    position: Position,
    size: Size,
}

impl<_Data> Node<_Data>
where
    _Data: Data,
{
    fn new(parent: Option<NodeKey>, size: Size, data: _Data) -> Self {
        Node {
            data,
            parent,
            children: Vec::new(),
            position: Position::default(),
            size,
        }
    }
}

/// Tree of layout objects. The tree is immutable, but the layout objects themselves are
/// mutable.
#[derive(Debug, Clone)]
struct Tree<_Data>
where
    _Data: Data,
{
    root: NodeKey,
    nodes: slotmap::SlotMap<NodeKey, Node<_Data>>,
}

impl<_Data: Data> Tree<_Data> {
    fn new(root_data: _Data, root_size: Size) -> Self {
        let mut nodes = slotmap::SlotMap::with_key();
        let root = Node::new(None /*parent*/, root_size, root_data);
        let root_key = nodes.insert(root);
        Tree {
            root: root_key,
            nodes,
        }
    }

    fn add_child(&mut self, parent: NodeKey, child: Node<_Data>) -> NodeKey {
        let child_key = self.nodes.insert(child);
        let mut parent_node = self.nodes.get_mut(parent).unwrap();
        parent_node.children.push(child_key);
        child_key
    }
}

/// is_leaf returns true if the node has no children.
fn is_leaf<_Data: Data>(nodekey: NodeKey, nodes: &slotmap::SlotMap<NodeKey, Node<_Data>>) -> bool {
    let node = nodes.get(nodekey).unwrap();
    node.children.is_empty()
}

fn is_leftmost_child<_Data: Data>(
    nodekey: NodeKey,
    nodes: &slotmap::SlotMap<NodeKey, Node<_Data>>,
) -> bool {
    let node = nodes.get(nodekey).unwrap();
    let parent = nodes.get(node.parent.unwrap()).unwrap();
    parent.children.first().unwrap() == &nodekey
}

fn is_rightmost_child<_Data: Data>(
    nodekey: NodeKey,
    nodes: &slotmap::SlotMap<NodeKey, Node<_Data>>,
) -> bool {
    let node = nodes.get(nodekey).unwrap();
    let parent = nodes.get(node.parent.unwrap()).unwrap();
    parent.children.last().unwrap() == &nodekey
}

fn get_previous_sibling<_Data: Data>(
    nodekey: NodeKey,
    nodes: &slotmap::SlotMap<NodeKey, Node<_Data>>,
) -> Option<NodeKey> {
    let node = nodes.get(nodekey).unwrap();
    let parent = nodes.get(node.parent.unwrap()).unwrap();
    let index = parent.children.iter().position(|&x| x == nodekey).unwrap();
    if index == 0 {
        None
    } else {
        Some(parent.children[index - 1])
    }
}

fn get_next_sibling<_Data: Data>(
    nodekey: NodeKey,
    nodes: &slotmap::SlotMap<NodeKey, Node<_Data>>,
) -> Option<NodeKey> {
    let node = nodes.get(nodekey).unwrap();
    let parent = nodes.get(node.parent.unwrap()).unwrap();
    let index = parent.children.iter().position(|&x| x == nodekey).unwrap();
    if index == parent.children.len() - 1 {
        None
    } else {
        Some(parent.children[index + 1])
    }
}

fn get_leftmost_sibling<_Data: Data>(
    nodekey: NodeKey,
    nodes: &slotmap::SlotMap<NodeKey, Node<_Data>>,
) -> NodeKey {
    let node = nodes.get(nodekey).unwrap();
    let parent = nodes.get(node.parent.unwrap()).unwrap();
    *parent.children.first().unwrap()
}

fn get_leftmost_child<_Data: Data>(
    nodekey: NodeKey,
    nodes: &slotmap::SlotMap<NodeKey, Node<_Data>>,
) -> NodeKey {
    let node = nodes.get(nodekey).unwrap();
    *node.children.first().unwrap()
}

fn get_rightmost_child<_Data: Data>(
    nodekey: NodeKey,
    nodes: &slotmap::SlotMap<NodeKey, Node<_Data>>,
) -> NodeKey {
    let node = nodes.get(nodekey).unwrap();
    *node.children.last().unwrap()
}

trait Layout<_Data>
where
    _Data: Data,
{
    fn layout(&self, tree: &mut Tree<_Data>);
    fn sibling_separation(&self) -> f64;
}

struct ReingoldTilfordLayout<_Data>
where
    _Data: Data,
{
    sibling_separation: f64,
    phantom_data: std::marker::PhantomData<_Data>,
}

impl<_Data: Data> ReingoldTilfordLayout<_Data> {
    fn new(sibling_separation: f64) -> Self {
        ReingoldTilfordLayout {
            sibling_separation,
            phantom_data: std::marker::PhantomData,
        }
    }

    // initialize x to -1, y to depth, and mod to 0 for each node. depth
    // is the depth of the node in the tree. The root node is at depth 0.
    fn initialize_nodes(&self, tree: &mut Tree<_Data>, depth: f64) {
        let mut stack = vec![(tree.root, depth)];
        while let Some((nodekey, depth)) = stack.pop() {
            let mut node = tree.nodes.get_mut(nodekey).unwrap();
            node.position.x = -1.0;
            node.position.y = depth;
            node.position.modifier = 0.0;
            for child in &node.children {
                stack.push((*child, depth + 1.0));
            }
        }
    }

    fn calculate_initial_x(&self, tree: &mut Tree<_Data>) {
        let mut stack = vec![tree.root];
        while let Some(nodekey) = stack.pop() {
            if is_leaf(nodekey, &tree.nodes) {
                if is_leftmost_child(nodekey, &tree.nodes) {
                    let node = tree.nodes.get_mut(nodekey).unwrap();
                    node.position.x = 0.0;
                } else {
                    let previous_sibling = get_previous_sibling(nodekey, &tree.nodes).unwrap();
                    let previous_sibling = tree.nodes.get(previous_sibling).unwrap();
                    let node = tree.nodes.get_mut(nodekey).unwrap();
                    node.position.x = previous_sibling.position.x + self.sibling_separation;
                }
            } else {
                let leftmost_child = get_leftmost_child(nodekey, &tree.nodes);
                let rightmost_child = get_rightmost_child(nodekey, &tree.nodes);
                let leftmost_child = tree.nodes.get(leftmost_child).unwrap();
                let rightmost_child = tree.nodes.get(rightmost_child).unwrap();
                let node = tree.nodes.get_mut(nodekey).unwrap();
                node.position.x = (leftmost_child.position.x + rightmost_child.position.x) / 2.0;
            }
            let node = tree.nodes.get(nodekey).unwrap();
            for child in &node.children {
                stack.push(*child);
            }
        }
    }
}

impl<_Data: Data> Layout<_Data> for ReingoldTilfordLayout<_Data> {
    fn layout(&self, tree: &mut Tree<_Data>) {
        self.initialize_nodes(tree, 0.0 /*depth*/);
        self.calculate_initial_x(tree);
    }

    fn sibling_separation(&self) -> f64 {
        self.sibling_separation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestNodeData {
        name: String,
    }

    impl Data for TestNodeData {}

    fn create_test_tree() -> Tree<TestNodeData> {
        let mut tree = Tree::new(
            TestNodeData {
                name: "root".to_string(),
            },
            Size {
                width: 100.0,
                height: 100.0,
            },
        );
        let root = tree.root;
        let child1 = tree.add_child(
            root,
            Node::new(
                Some(root),
                Size {
                    width: 100.0,
                    height: 100.0,
                },
                TestNodeData {
                    name: "child1".to_string(),
                },
            ),
        );
        let child2 = tree.add_child(
            root,
            Node::new(
                Some(root),
                Size {
                    width: 100.0,
                    height: 100.0,
                },
                TestNodeData {
                    name: "child2".to_string(),
                },
            ),
        );
        let child3 = tree.add_child(
            root,
            Node::new(
                Some(root),
                Size {
                    width: 100.0,
                    height: 100.0,
                },
                TestNodeData {
                    name: "child3".to_string(),
                },
            ),
        );

        let child3_1 = tree.add_child(
            child3,
            Node::new(
                Some(child3),
                Size {
                    width: 100.0,
                    height: 100.0,
                },
                TestNodeData {
                    name: "child3_1".to_string(),
                },
            ),
        );
        let child3_2 = tree.add_child(
            child3,
            Node::new(
                Some(child3),
                Size {
                    width: 100.0,
                    height: 100.0,
                },
                TestNodeData {
                    name: "child3_2".to_string(),
                },
            ),
        );
        let child3_3 = tree.add_child(
            child3,
            Node::new(
                Some(child3),
                Size {
                    width: 100.0,
                    height: 100.0,
                },
                TestNodeData {
                    name: "child3_3".to_string(),
                },
            ),
        );

        tree
    }

    // test you can create a tree with a root node and three children.
    #[test]
    fn test_create_tree() {
        let tree = create_test_tree();
        let root = tree.root;
        let nodes = &tree.nodes;
        let root_node = nodes.get(root).unwrap();
        assert_eq!(root_node.data.name, "root");
        assert_eq!(root_node.children.len(), 3);
    }

    // test that in the test tree, the first two children are leaves, the third child is not, and the root is not.
    #[test]
    fn test_is_leaf() {
        let tree = create_test_tree();
        let root = tree.root;
        let nodes = &tree.nodes;
        let root_node = nodes.get(root).unwrap();
        assert!(!is_leaf(root, nodes));
        assert!(is_leaf(root_node.children[0], nodes));
        assert!(is_leaf(root_node.children[1], nodes));
        assert!(!is_leaf(root_node.children[2], nodes));
    }

    // test that only the first child of the root is the leftmost child.
    #[test]
    fn test_is_leftmost_child() {
        let tree = create_test_tree();
        let root = tree.root;
        let nodes = &tree.nodes;
        let root_node = nodes.get(root).unwrap();
        assert!(is_leftmost_child(root_node.children[0], nodes));
        assert!(!is_leftmost_child(root_node.children[1], nodes));
        assert!(!is_leftmost_child(root_node.children[2], nodes));
    }

    // test that only the last child of the root is the rightmost child.
    #[test]
    fn test_is_rightmost_child() {
        let tree = create_test_tree();
        let root = tree.root;
        let nodes = &tree.nodes;
        let root_node = nodes.get(root).unwrap();
        assert!(!is_rightmost_child(root_node.children[0], nodes));
        assert!(!is_rightmost_child(root_node.children[1], nodes));
        assert!(is_rightmost_child(root_node.children[2], nodes));
    }

    // test that get_previous_sibling returns the correct sibling.
    #[test]
    fn test_get_previous_sibling() {
        let tree = create_test_tree();
        let root = tree.root;
        let nodes = &tree.nodes;
        let root_node = nodes.get(root).unwrap();
        assert_eq!(get_previous_sibling(root_node.children[0], nodes), None);
        assert_eq!(
            get_previous_sibling(root_node.children[1], nodes),
            Some(root_node.children[0])
        );
        assert_eq!(
            get_previous_sibling(root_node.children[2], nodes),
            Some(root_node.children[1])
        );
    }

    // test that get_next_sibling returns the correct sibling.
    #[test]
    fn test_get_next_sibling() {
        let tree = create_test_tree();
        let root = tree.root;
        let nodes = &tree.nodes;
        let root_node = nodes.get(root).unwrap();
        assert_eq!(
            get_next_sibling(root_node.children[0], nodes),
            Some(root_node.children[1])
        );
        assert_eq!(
            get_next_sibling(root_node.children[1], nodes),
            Some(root_node.children[2])
        );
        assert_eq!(get_next_sibling(root_node.children[2], nodes), None);
    }

    // test get_leftmost_sibling returns the leftmost sibling.
    #[test]
    fn test_get_leftmost_sibling() {
        let tree = create_test_tree();
        let root = tree.root;
        let nodes = &tree.nodes;
        let root_node = nodes.get(root).unwrap();
        assert_eq!(
            get_leftmost_sibling(root_node.children[0], nodes),
            root_node.children[0]
        );
        assert_eq!(
            get_leftmost_sibling(root_node.children[1], nodes),
            root_node.children[0]
        );
        assert_eq!(
            get_leftmost_sibling(root_node.children[2], nodes),
            root_node.children[0]
        );
    }

    // test get_leftmost_child returns the leftmost child.
    #[test]
    fn test_get_leftmost_child() {
        let tree = create_test_tree();
        let root = tree.root;
        let nodes = &tree.nodes;
        let root_node = nodes.get(root).unwrap();
        assert_eq!(get_leftmost_child(root, nodes), root_node.children[0]);
    }

    // test get_rightmost_child returns the rightmost child.
    #[test]
    fn test_get_rightmost_child() {
        let tree = create_test_tree();
        let root = tree.root;
        let nodes = &tree.nodes;
        let root_node = nodes.get(root).unwrap();
        assert_eq!(get_rightmost_child(root, nodes), root_node.children[2]);
    }

    // test initialize_nodes initializes the nodes correctly. For all nodex X == -1, mod == 0,
    // and the depth is correct.
    #[test]
    fn test_reingold_tilford_initialize_nodes() {
        let mut tree = create_test_tree();
        let sibling_distance = 1.0;
        let layout = ReingoldTilfordLayout::new(sibling_distance);
        layout.initialize_nodes(&mut tree, 0.0 /*depth*/);

        let root = tree.root;
        let nodes = &tree.nodes;
        let root_node = nodes.get(root).unwrap();
        assert_eq!(root_node.position.x, -1.0);
        assert_eq!(root_node.position.modifier, 0.0);
        assert_eq!(root_node.position.y, 0.0);
        for child in root_node.children.iter() {
            let child_node = nodes.get(*child).unwrap();
            assert_eq!(child_node.position.x, -1.0);
            assert_eq!(child_node.position.modifier, 0.0);
            assert_eq!(child_node.position.y, 1.0);
        }
    }
}
