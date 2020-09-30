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
    fn find<'a>(&'a self, document: &'a Value) -> Result<Vec<&'a Value>, FindError>;
}

struct SelectorPath {
    matchers: Vec<Box<dyn matchers::Matcher>>,
}

pub fn new<'a>(matchers: Vec<Box<dyn matchers::Matcher>>) -> impl Path + 'a {
    SelectorPath { matchers }
}

impl Path for SelectorPath {
    fn find<'a>(&'a self, document: &'a Value) -> Result<Vec<&'a Value>, FindError> {
        // pass nodes, starting with document alone, through each matcher in turn
        Ok((&self.matchers)
            .iter()
            .fold(vec![document], |nodes, matcher| {
                nodes.iter().flat_map(|node| matcher.select(node)).collect()
            }))
    }
}
