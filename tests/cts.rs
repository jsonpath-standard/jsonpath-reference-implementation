/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

#[cfg(test)]
mod tests {
    use jsonpath_reference_implementation::jsonpath;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::panic;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestSuite {
        tests: Vec<Testcase>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Testcase {
        name: String,
        selector: String,

        #[serde(default)]
        invalid_selector: bool,

        #[serde(default)]
        document: serde_json::Value, // omitted if invalid_selector = true

        #[serde(default)]
        result: serde_json::Value, // omitted if invalid_selector = true

        #[serde(default)]
        focus: bool, // if true, run only tests with focus set to true
    }

    #[test]
    fn compliance_test_suite() {
        let cts_json = fs::read_to_string("tests/cts.json").expect("failed to read cts.json");

        let suite: TestSuite =
            serde_json::from_str(&cts_json).expect("failed to deserialize cts.json");

        let focussed = (&suite.tests).iter().find(|t| t.focus).is_some();

        let mut errors: Vec<String> = Vec::new();
        for t in suite.tests {
            if focussed && !t.focus {
                continue;
            }
            let result = panic::catch_unwind(|| {
                if t.invalid_selector {
                    println!(
                        "testcase name = `{}`, selector = `{}`, expected invalid selector.",
                        t.name, t.selector
                    );
                } else {
                    println!(
                        "testcase name = `{}`, selector = `{}`, document:\n{:#}\nexpected result = `{}`.",
                        t.name, t.selector, t.document, t.result
                    );
                }
                let path = jsonpath::parse(&t.selector);

                if let Ok(ref p) = path {
                    if t.invalid_selector {
                        assert!(
                            path.is_err(),
                            "{}: parsing {} should have failed",
                            t.name,
                            t.selector
                        );
                    }
                    if let Ok(result) = p.find(&t.document) {
                        if !equal(&result, as_array(&t.result).expect("invalid result")) {
                            assert!(
                                false,
                                "incorrect result for {}, expected: {:?}, got: {:?}",
                                t.name,
                                as_array(&t.result).unwrap(),
                                result
                            )
                        }
                    } else {
                        assert!(false, "find failed") // should not happen
                    }
                } else {
                    if !t.invalid_selector {
                        assert!(
                            path.is_ok(),
                            "{}: parsing {} should have succeeded but failed: {}",
                            t.name,
                            t.selector,
                            path.err().expect("should be an error")
                        );
                    }
                }
            });
            if let Err(err) = result {
                errors.push(format!("{:?}", err));
            }
        }
        assert!(errors.is_empty());
        if focussed {
            assert!(false, "testcase(s) still focussed")
        }
    }

    fn equal(actual: &Vec<&serde_json::Value>, expected: Vec<serde_json::Value>) -> bool {
        if actual.len() != expected.len() {
            false
        } else {
            (0..actual.len()).fold(true, |result, item| {
                result && actual[item] == &expected[item]
            })
        }
    }

    fn as_array(v: &serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
        match v {
            serde_json::Value::Array(seq) => {
                let array_elements = seq.into_iter().map(|v| v.clone());
                Ok(array_elements.collect())
            }
            _ => Err("not a sequence".to_string()),
        }
    }
}
