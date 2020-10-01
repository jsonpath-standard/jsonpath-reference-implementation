/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

use serde_json::Value;
use std::iter;

/// An iterator over matcher selection results.
type Iter<'a> = Box<dyn Iterator<Item = &'a Value> + 'a>;

/// Matcher maps a node to a list of nodes. If the input node is not matched by the matcher or
/// the matcher does not select any subnodes of the input node, then the result is empty.
pub trait Matcher {
    fn select<'a>(&'a self, node: &'a Value) -> Iter<'a>;
}

/// Selects exactly one item, namely the node
/// of the subtree the selector is applied to.
///
/// (which may or may be not the actual root of the document).
pub struct RootSelector {}

impl Matcher for RootSelector {
    fn select<'a>(&self, node: &'a Value) -> Iter<'a> {
        Box::new(iter::once(node))
    }
}

/// Selects all children of a node.
pub struct WildcardedChild {}

impl Matcher for WildcardedChild {
    fn select<'a>(&self, node: &'a Value) -> Iter<'a> {
        match node {
            Value::Object(m) => Box::new(m.values()),
            Value::Array(a) => Box::new(a.iter()),
            _ => Box::new(iter::empty()),
        }
    }
}

/// Selects a named child.
pub struct Child {
    name: String,
}

impl Child {
    pub fn new(name: String) -> Self {
        Child { name }
    }
}

impl Matcher for Child {
    fn select<'a>(&self, node: &'a Value) -> Iter<'a> {
        Box::new(node.get(&self.name).into_iter())
    }
}

/// Selects an array item by index.
///
/// If the index is negative, it references element len-abs(index).
pub struct ArrayIndex {
    index: i64,
}

impl ArrayIndex {
    pub fn new(index: i64) -> Self {
        ArrayIndex { index }
    }
}

impl Matcher for ArrayIndex {
    fn select<'a>(&self, node: &'a Value) -> Iter<'a> {
        let len = if let Value::Array(a) = node {
            a.len()
        } else {
            0
        };
        let idx = if self.index >= 0 {
            self.index as usize
        } else {
            let abs = (-self.index) as usize;
            if abs < len {
                len - abs
            } else {
                return Box::new(iter::empty());
            }
        };
        Box::new(node.get(idx).into_iter())
    }
}

/// Applies a sequence of selectors on the same node and returns
/// a concatenation of the results.
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn object_wildcard() {
        let s = WildcardedChild {};
        let j = json!({"a": 1, "b": 2});
        let r: Vec<&Value> = s.select(&j).collect();
        assert_eq!(format!("{:?}", r), "[Number(1), Number(2)]");
    }

    #[test]
    fn array_wildcard() {
        let s = WildcardedChild {};
        let j = json!([1, 2]);
        let r: Vec<&Value> = s.select(&j).collect();
        assert_eq!(format!("{:?}", r), "[Number(1), Number(2)]");
    }

    #[test]
    fn array_index() {
        let s = ArrayIndex::new(1);
        let j = json!([1, 2]);
        let r: Vec<&Value> = s.select(&j).collect();
        assert_eq!(format!("{:?}", r), "[Number(2)]");
    }

    #[test]
    fn array_index_zero() {
        let s = ArrayIndex::new(0);
        let j = json!([1, 2]);
        let r: Vec<&Value> = s.select(&j).collect();
        assert_eq!(format!("{:?}", r), "[Number(1)]");
    }

    #[test]
    fn array_index_oob() {
        let s = ArrayIndex::new(4);
        let j = json!([1, 2]);
        let r: Vec<&Value> = s.select(&j).collect();
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn array_index_negative() {
        let s = ArrayIndex::new(-1);
        let j = json!([1, 2]);
        let r: Vec<&Value> = s.select(&j).collect();
        assert_eq!(format!("{:?}", r), "[Number(2)]");
    }

    #[test]
    fn array_index_negative_oob() {
        let s = ArrayIndex::new(-10);
        let j = json!([1, 2]);
        let r: Vec<&Value> = s.select(&j).collect();
        assert_eq!(r.len(), 0);
    }
}
