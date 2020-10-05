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

pub fn parse(selector: &str) -> Result<impl path::Path, String> {
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

    Ok(path::new(ms))
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
                ms.push(Box::new(matchers::Child::new(unescape_single(r.as_str()))));
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

fn unescape(contents: &str) -> String {
    let s = format!(r#""{}""#, contents);
    serde_json::from_str(&s).unwrap()
}

fn unescape_single(contents: &str) -> String {
    let d = to_double_quoted(contents);
    unescape(&d)
}

// converts a single quoted string body into a string that can be unescaped
// by a function that knows how to unescape double quoted string,
// It works by unescaping single quotes and escaping double quotes while leaving
// everything else untouched.
fn to_double_quoted(contents: &str) -> String {
    let mut output = String::new();
    let mut escaping = false;
    for ch in contents.chars() {
        if !escaping {
            if ch == '\\' {
                escaping = true;
            } else {
                if ch == '"' {
                    output.push('\\');
                }
                output.push(ch);
            }
        } else {
            escaping = false;
            if ch != '\'' {
                output.push('\\');
            };
            output.push(ch);
        }
    }
    output
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_double() {
        assert_eq!(to_double_quoted(r#"ab"#), r#"ab"#);
        assert_eq!(to_double_quoted(r#"a"b"#), r#"a\"b"#);
        assert_eq!(to_double_quoted(r#"a\'b"#), r#"a'b"#);
        assert_eq!(to_double_quoted(r#"a\nb"#), r#"a\nb"#);
        assert_eq!(to_double_quoted(r#"a\bb"#), r#"a\bb"#);
        assert_eq!(to_double_quoted(r#"a\\b"#), r#"a\\b"#);
    }
}
