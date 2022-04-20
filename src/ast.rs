/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

use serde_json::Value;
use slyce::Slice;
use std::iter;

/// A path is a tree of selector nodes.
///
/// For example, the JSONPath `$.foo.bar` yields this AST:
///
/// ```text
///              ^
///             / \
///            ^   \___ DotName("bar")
///           / \
/// Root ___ /   \___ DotName("foo")
/// ```
///
/// A more complicated example: `$.foo[1,2]["bar"]`:
///
/// ```text
///                 ^
///                / \
///               ^   \___ Union
///              / \            \
///             /   \___ Union   \___ [Name("bar")]
///            /              \
///           ^                \___ [Index(1), Index(2)]
///          / \
/// Root ___/   \___ DotName("foo")
/// ```
///
/// Selectors are left associative, thus `$.foo[1,2]["bar"]` behaves
/// like (pseudocode) `(($.foo)[1,2])["bar"]`; thus the root of the resulting
/// tree is actually the right-most selector (the last one to be applied).
///
/// The Path::Root AST node is called "root" because that's the
/// name of the node in the JSONPath grammar. It represents the source of
/// the json value stream which gets operated upon by Selector nodes.
/// This is why despite being called "root", this node doesn't lie at the root
/// of the AST tree.
#[derive(Debug)]
pub enum Path {
    Root,
    Sel(Box<Path>, Selector),
}

#[derive(Debug)]
pub enum Selector {
    Union(Vec<UnionElement>),
    DotName(String),
    DotWildcard,
}

#[derive(Debug)]
pub enum UnionElement {
    Name(String),
    Slice(Slice),
    Index(i64),
}

type Iter<'a> = Box<dyn Iterator<Item = &'a Value> + 'a>;

impl Path {
    pub fn find<'a>(&'a self, input: &'a Value) -> Iter<'a> {
        match self {
            Path::Root => Box::new(std::iter::once(input)),
            Path::Sel(left, sel) => Box::new(left.find(input).flat_map(move |v| sel.find(v))),
        }
    }
}

impl Selector {
    pub fn find<'a>(&'a self, input: &'a Value) -> Iter<'a> {
        match self {
            Selector::Union(indices) => Box::new(indices.iter().flat_map(move |i| i.find(input))),
            Selector::DotName(name) => Box::new(input.get(name).into_iter()),
            Selector::DotWildcard => match input {
                Value::Object(m) => Box::new(m.values()),
                Value::Array(a) => Box::new(a.iter()),
                _ => Box::new(std::iter::empty()),
            },
        }
    }
}

impl UnionElement {
    pub fn find<'a>(&self, v: &'a Value) -> Iter<'a> {
        match self {
            UnionElement::Name(name) => Box::new(v.get(name).into_iter()),
            UnionElement::Slice(slice) => {
                if let Value::Array(arr) = v {
                    Box::new(slice.apply(arr))
                } else {
                    Box::new(iter::empty())
                }
            }
            UnionElement::Index(num) => Box::new(v.get(abs_index(*num, v)).into_iter()),
        }
    }
}

fn abs_index(index: i64, node: &Value) -> usize {
    if index >= 0 {
        index as usize
    } else {
        let len = if let Value::Array(a) = node {
            a.len() as i64
        } else {
            0
        };
        (len + index) as usize
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::parse;
    use serde_json::json;

    #[test]
    fn demo() {
        let a1 = Path::Sel(Box::new(Path::Root), Selector::DotName("foo".to_owned()));
        let a2 = Path::Sel(Box::new(a1), Selector::DotName("bar".to_owned()));
        let a3 = Path::Sel(
            Box::new(a2),
            Selector::Union(vec![UnionElement::Name("baz".to_owned())]),
        );
        let a4 = Path::Sel(Box::new(a3), Selector::Union(vec![UnionElement::Index(4)]));

        let j = json!({"foo":{"bar":{"baz":[10,20,30,40,50,60]}}});
        println!("j: {}", j);

        let v = a4.find(&j).collect::<Vec<_>>();
        assert_eq!(v[0], 50);
    }

    #[test]
    fn parse_demo() -> Result<(), String> {
        let p = parse("$.foo['bar'].*[4,-1]")?;
        println!("AST: {:?}", &p);
        let j = json!({"foo":{"bar":{"baz":[10,20,30,40,50,60]}}});

        let v = p.find(&j).collect::<Vec<_>>();
        println!("RES: {:?}", v);

        assert_eq!(v[0], 50);
        assert_eq!(v[1], 60);
        Ok(())
    }
}
