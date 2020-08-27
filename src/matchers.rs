/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

use serde_json::Value;

// Matcher maps a node to a list of nodes. If the input node is not matched by the matcher or
// the matcher does not select any subnodes of the input node, then the result is empty.
pub trait Matcher {
    fn select<'a>(&self, node: &'a Value) -> Vec<&'a Value>;
}

pub struct RootSelector {}

impl Matcher for RootSelector {
    fn select<'a>(&self, node: &'a Value) -> Vec<&'a Value> {
        let mut results = Vec::new();
        results.push(node);
        results
    }
}
