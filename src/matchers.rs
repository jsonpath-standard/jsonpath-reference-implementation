/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

use serde_json::Value;
use std::iter;

/// An iterator over matcher selection results.
type Iter<'a> = Box<dyn Iterator<Item = &'a Value> + 'a>;

// Matcher maps a node to a list of nodes. If the input node is not matched by the matcher or
// the matcher does not select any subnodes of the input node, then the result is empty.
pub trait Matcher {
    fn select<'a>(&'a self, node: &'a Value) -> Iter<'a>;
}

pub struct RootSelector {}

impl Matcher for RootSelector {
    fn select<'a>(&'a self, node: &'a Value) -> Iter<'a> {
        Box::new(iter::once(node))
    }
}

pub struct WildcardedChild {}

impl Matcher for WildcardedChild {
    fn select<'a>(&self, node: &'a Value) -> Iter<'a> {
        if let Some(m) = node.as_object() {
            Box::new(m.values())
        } else {
            Box::new(iter::empty())
        }
    }
}

pub struct Child {
    name: String,
}

impl Child {
    pub fn new(name: String) -> Self {
        Child { name }
    }
}

impl Matcher for Child {
    fn select<'a>(&'a self, node: &'a Value) -> Iter<'a> {
        Box::new(node.get(&self.name).into_iter())
    }
}

pub struct Union {
    elements: Vec<Box<dyn Matcher>>,
}

impl Union {
    pub fn new(elements: Vec<Box<dyn Matcher>>) -> Self {
        Union { elements }
    }
}

impl Matcher for Union {
    fn select<'a>(&'a self, node: &'a Value) -> Iter<'a> {
        Box::new(self.elements.iter().flat_map(move |it| it.select(node)))
    }
}
