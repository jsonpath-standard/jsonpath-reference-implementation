/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

pub use crate::ast::*;
use crate::pest::Parser;
use slyce::{Index, Slice};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct PathParser;

pub fn parse(selector: &str) -> Result<Path, String> {
    let selector_rule = PathParser::parse(Rule::selector, selector)
        .map_err(|e| format!("{}", e))?
        .nth(1)
        .unwrap();

    Ok(selector_rule
        .into_inner()
        .fold(Path::Root, |prev, r| match r.as_rule() {
            Rule::matcher => Path::Sel(Box::new(prev), parse_selector(r)),
            _ => panic!("invalid parse tree {:?}", r),
        }))
}

fn parse_selector(matcher_rule: pest::iterators::Pair<Rule>) -> Selector {
    let r = matcher_rule.into_inner().next().unwrap();

    match r.as_rule() {
        Rule::wildcardedDotChild => Selector::DotWildcard,
        Rule::namedDotChild => Selector::DotName(parse_child_name(r)),
        Rule::union => Selector::Union(parse_union_indices(r)),
        _ => panic!("invalid parse tree {:?}", r),
    }
}

fn parse_child_name(matcher_rule: pest::iterators::Pair<Rule>) -> String {
    let r = matcher_rule.into_inner().next().unwrap();

    match r.as_rule() {
        Rule::childName => r.as_str().to_owned(),
        _ => panic!("invalid parse tree {:?}", r),
    }
}

fn parse_union_indices(matcher_rule: pest::iterators::Pair<Rule>) -> Vec<UnionElement> {
    matcher_rule
        .into_inner()
        .map(|r| match r.as_rule() {
            Rule::unionChild => parse_union_child(r),
            Rule::unionArraySlice => parse_union_array_slice(r),
            Rule::unionArrayIndex => parse_union_array_index(r),
            _ => panic!("invalid parse tree {:?}", r),
        })
        .collect()
}

fn parse_union_child(matcher_rule: pest::iterators::Pair<Rule>) -> UnionElement {
    let r = matcher_rule.into_inner().next().unwrap();

    UnionElement::Name(match r.as_rule() {
        Rule::doubleInner => unescape(r.as_str()),
        Rule::singleInner => unescape_single(r.as_str()),
        _ => panic!("invalid parse tree {:?}", r),
    })
}

fn parse_union_array_index(matcher_rule: pest::iterators::Pair<Rule>) -> UnionElement {
    let i = matcher_rule.as_str().parse().unwrap();
    UnionElement::Index(i)
}

fn parse_union_array_slice(matcher_rule: pest::iterators::Pair<Rule>) -> UnionElement {
    let mut start: Option<isize> = None;
    let mut end: Option<isize> = None;
    let mut step: Option<isize> = None;
    for r in matcher_rule.into_inner() {
        match r.as_rule() {
            Rule::sliceStart => {
                start = Some(r.as_str().parse().unwrap());
            }

            Rule::sliceEnd => {
                end = Some(r.as_str().parse().unwrap());
            }

            Rule::sliceStep => {
                step = Some(r.as_str().parse().unwrap());
            }

            _ => panic!("invalid parse tree {:?}", r),
        }
    }

    UnionElement::Slice(Slice {
        start: start.map(Index::from).unwrap_or_default(),
        end: end.map(Index::from).unwrap_or_default(),
        step,
    })
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
