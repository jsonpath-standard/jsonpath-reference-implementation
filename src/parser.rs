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

pub fn parse<'a>(selector: &'a str) -> Result<Box<dyn path::Path + 'a>, String> {
    let selector_rule = PathParser::parse(Rule::selector, selector)
        .map_err(|e| format!("{}", e))?
        .next()
        .unwrap();

    let mut ms: Vec<Box<dyn matchers::Matcher>> = Vec::new();
    for r in selector_rule.into_inner() {
        match r.as_rule() {
            Rule::rootSelector => ms.push(Box::new(matchers::RootSelector {})),
            Rule::matcher => {
                for m in parse_matcher(r) {
                    ms.push(m)
                }
            }
            _ => println!("r={:?}", r),
        }
    }

    Ok(Box::new(path::new(ms)))
}

fn parse_matcher(matcher_rule: pest::iterators::Pair<Rule>) -> Vec<Box<dyn matchers::Matcher>> {
    let mut ms: Vec<Box<dyn matchers::Matcher>> = Vec::new();
    for r in matcher_rule.into_inner() {
        match r.as_rule() {
            Rule::wildcardedDotChild => ms.push(Box::new(matchers::WildcardedChild {})),
            Rule::namedDotChild => {
                for m in parse_dot_child_matcher(r) {
                    ms.push(m)
                }
            }
            _ => (),
        }
    }
    ms
}

fn parse_dot_child_matcher(
    matcher_rule: pest::iterators::Pair<Rule>,
) -> Vec<Box<dyn matchers::Matcher>> {
    let mut ms: Vec<Box<dyn matchers::Matcher>> = Vec::new();
    for r in matcher_rule.into_inner() {
        if let Rule::childName = r.as_rule() {
            ms.push(Box::new(matchers::new_dot_child_matcher(
                r.as_str().to_owned(),
            )));
        }
    }
    ms
}
