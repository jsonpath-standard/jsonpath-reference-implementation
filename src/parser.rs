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

pub fn parse(selector: &str) -> Result<Box<dyn path::Path + '_>, String> {
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

            Rule::union => {
                for m in parse_union(r) {
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
            ms.push(Box::new(matchers::Child::new(r.as_str().to_owned())));
        }
    }
    ms
}

fn parse_union(matcher_rule: pest::iterators::Pair<Rule>) -> Vec<Box<dyn matchers::Matcher>> {
    let mut ms: Vec<Box<dyn matchers::Matcher>> = Vec::new();
    for r in matcher_rule.into_inner() {
        match r.as_rule() {
            Rule::unionChild => {
                for m in parse_union_child(r) {
                    ms.push(m)
                }
            }
            Rule::unionArrayIndex => {
                for m in parse_union_array_index(r) {
                    ms.push(m)
                }
            }
            _ => {}
        }
    }
    vec![Box::new(matchers::Union::new(ms))]
}

fn parse_union_child(matcher_rule: pest::iterators::Pair<Rule>) -> Vec<Box<dyn matchers::Matcher>> {
    let mut ms: Vec<Box<dyn matchers::Matcher>> = Vec::new();
    for r in matcher_rule.into_inner() {
        match r.as_rule() {
            Rule::doubleInner => {
                ms.push(Box::new(matchers::Child::new(unescape(r.as_str()))));
            }

            Rule::singleInner => {
                ms.push(Box::new(matchers::Child::new(unescape(r.as_str()))));
            }

            _ => (),
        }
    }
    ms
}

fn parse_union_array_index(
    matcher_rule: pest::iterators::Pair<Rule>,
) -> Vec<Box<dyn matchers::Matcher>> {
    let mut ms: Vec<Box<dyn matchers::Matcher>> = Vec::new();
    let i = matcher_rule.as_str().parse().unwrap();
    ms.push(Box::new(matchers::ArrayIndex::new(i)));
    ms
}

const ESCAPED: &str = "\"'\\/bfnrt";
const UNESCAPED: &str = "\"'\\/\u{0008}\u{000C}\u{000A}\u{000D}\u{0009}";

fn unescape(contents: &str) -> String {
    let mut output = String::new();
    let xs: Vec<char> = contents.chars().collect();
    let mut i = 0;
    while i < xs.len() {
        if xs[i] == '\\' {
            i += 1;
            if xs[i] == 'u' {
                i += 1;

                // convert xs[i..i+4] to Unicode character and add it to the output
                let x = xs[i..i + 4].iter().collect::<String>();
                let n = u32::from_str_radix(&x, 16);
                let u = std::char::from_u32(n.unwrap());
                output.push(u.unwrap());

                i += 4;
            } else {
                for (j, c) in ESCAPED.chars().enumerate() {
                    if xs[i] == c {
                        output.push(UNESCAPED.chars().nth(j).unwrap())
                    }
                }
                i += 1;
            }
        } else {
            output.push(xs[i]);
            i += 1;
        }
    }
    output
}
