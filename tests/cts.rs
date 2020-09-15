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

    const VERBOSE: bool = false;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestSuite {
        tests: Vec<Testcase>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Testcase {
        name: String,
        selector: String,
        document: serde_json::Value,
        result: serde_json::Value,
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
                if VERBOSE {
                    println!(
                        "name = {}, selector = {}, document = {:?}, result = {:?}",
                        t.name, t.selector, t.document, t.result
                    );
                }
                let path = jsonpath::parse(&t.selector);
                assert!(
                    path.is_ok(),
                    "parsing {} failed: {}",
                    t.selector,
                    path.err().expect("should be an error")
                );

                if let Ok(p) = path {
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
