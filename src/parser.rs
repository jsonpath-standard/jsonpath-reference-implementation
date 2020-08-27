/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

use crate::matchers;
use crate::path::{FindError, Path};
use crate::pest::Parser;
use serde_json::Value;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct PathParser;

struct SelectorPath<'a> {
    matchers: Vec<&'a dyn matchers::Matcher>,
}

#[allow(clippy::single_match)]
pub fn parse(selector: &str) -> Result<Box<dyn Path>, String> {
    let selector_rule = PathParser::parse(Rule::selector, selector)
        .map_err(|e| format!("{}", e))?
        .next()
        .unwrap();

    let mut ms: Vec<&dyn matchers::Matcher> = Vec::new();
    for r in selector_rule.into_inner() {
        match r.as_rule() {
            Rule::rootSelector => {
                ms.push(&matchers::RootSelector {});
            }
            _ => println!("r={:?}", r),
        }
    }

    Ok(Box::new(SelectorPath { matchers: ms }))
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
