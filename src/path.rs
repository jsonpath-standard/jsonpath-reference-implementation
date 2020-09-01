/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

use crate::matchers;
use serde_json::Value;

pub enum FindError {
    // no errors yet
}

pub trait Path {
    fn find<'a>(&self, document: &'a Value) -> Result<Vec<&'a Value>, FindError>;
}

struct SelectorPath<'a> {
    matchers: Vec<&'a dyn matchers::Matcher>,
}

pub fn new<'a>(matchers: Vec<&'a dyn matchers::Matcher>) -> impl Path + 'a {
    SelectorPath { matchers }
}

impl Path for SelectorPath<'_> {
    fn find<'a>(&self, document: &'a Value) -> Result<Vec<&'a Value>, FindError> {
        // pass nodes, starting with document alone, through each matcher in turn
        Ok((&self.matchers)
            .iter()
            .fold(doc_node(document), |nodes, matcher| {
                nodes.iter().flat_map(|node| matcher.select(node)).collect()
            }))
    }
}

fn doc_node<'a>(document: &'a Value) -> Vec<&'a Value> {
    let mut nodes = Vec::new();
    nodes.push(document);
    nodes
}
