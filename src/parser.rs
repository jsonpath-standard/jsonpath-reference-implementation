/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

use crate::matchers;
use crate::path;
use crate::pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct PathParser;

pub fn parse(selector: &str) -> Result<Box<dyn path::Path>, String> {
    let selector_rule = PathParser::parse(Rule::selector, selector)
        .map_err(|e| format!("{}", e))?
        .next()
        .unwrap();

    let mut ms: Vec<&dyn matchers::Matcher> = Vec::new();
    for r in selector_rule.into_inner() {
        match r.as_rule() {
            Rule::rootSelector => ms.push(&matchers::RootSelector {}),
            Rule::matcher => parse_matcher(r, &mut ms),
            _ => println!("r={:?}", r),
        }
    }

    Ok(Box::new(path::new(ms)))
}

fn parse_matcher(matcher_rule: pest::iterators::Pair<Rule>, ms: &mut Vec<&dyn matchers::Matcher>) {
    for r in matcher_rule.into_inner() {
        if let Rule::wildcardedDotChild = r.as_rule() {
            ms.push(&matchers::WildcardedChild {});
        }
    }
}
