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
use std::rc::Weak;

pub trait Data: Debug + Clone {}

/// Node in a tree of layout objects. Each node has a parent (except the root) and children.
/// The parent is a weak reference to avoid a reference cycle. The children are stored in a
/// `RefCell` to allow for mutation even when there are other references to the node. This is
/// necessary because the layout tree is mutated during the layout process.
#[derive(Debug, Clone)]
struct Node<_Data>
where
    _Data: Data,
{
    data: _Data,
    parent: Option<Weak<RefCell<Node<_Data>>>>,
    children: Vec<RefCell<Node<_Data>>>,
}

/// Tree of layout objects. The tree is immutable, but the layout objects themselves are
/// mutable.
#[derive(Debug, Clone)]
struct Tree<_Data>
where
    _Data: Data,
{
    root: RefCell<Node<_Data>>,
}

trait Layout<_Data>
where
    _Data: Data,
{
    fn layout(&self, tree: &mut Tree<_Data>);
}

struct ReingoldTilfordLayout<_Data>
where
    _Data: Data,
{
    phantom_data: std::marker::PhantomData<_Data>,
}

impl<_Data> Layout<_Data> for ReingoldTilfordLayout<_Data>
where
    _Data: Data,
{
    fn layout(&self, _tree: &mut Tree<_Data>) {}
}
