/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

#[cfg(test)]
mod tests {
    use jsonpath_ri::jsonpath;
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
        document: serde_yaml::Value, // JSON deserialised as YAML
        result: serde_yaml::Value,   // JSON deserialised as YAML
    }

    #[test]
    fn compliance_test_suite() {
        let y = fs::read_to_string("tests/cts.yaml").expect("failed to read cts.yaml");

        let suite: TestSuite = serde_yaml::from_str(&y).expect("failed to deserialize cts.yaml");

        let mut errors: Vec<String> = Vec::new();
        for t in suite.tests {
            let result = panic::catch_unwind(|| {
                println!(
                    "name = {}, selector = {}, document = {}, result = {}",
                    t.name,
                    t.selector,
                    as_json(&t.document).expect("invalid document"),
                    as_json(&t.result).expect("invalid result")
                );
                let path = jsonpath::parse(&t.selector);
                assert!(
                    path.is_ok(),
                    "parsing {} failed: {}",
                    t.selector,
                    path.err().expect("should be an error")
                );

                if let Ok(p) = path {
                    if let Ok(result) =
                        p.find(as_json_value(&t.document).expect("invalid document"))
                    {
                        if result != as_json_value_array(&t.result).expect("invalid result") {
                            assert!(
                                false,
                                "incorrect result, expected: {:?}, got: {:?}",
                                as_json_value_array(&t.result).unwrap(),
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
        assert!(errors.is_empty())
    }

    fn as_json(v: &serde_yaml::Value) -> Result<String, String> {
        match v {
            serde_yaml::Value::Null => Ok("null".to_string()),

            serde_yaml::Value::Bool(b) => Ok(b.to_string()),

            serde_yaml::Value::Number(num) => Ok(num.to_string()),

            serde_yaml::Value::String(s) => Ok(json::stringify(s.to_string())),

            serde_yaml::Value::Sequence(seq) => {
                let array_elements = seq
                    .into_iter()
                    .map(|v| as_json(v).expect("invalid sequence element"));
                Ok(format!("[{}]", itertools::join(array_elements, ",")))
            }

            serde_yaml::Value::Mapping(map) => {
                let object_members = map.iter().map(|(k, v)| {
                    format!(
                        "{}:{}",
                        as_json(k).expect("invalid object key"),
                        as_json(v).expect("invalid object value")
                    )
                });
                Ok(format!("{{{}}}", itertools::join(object_members, ",")))
            }
        }
    }

    fn as_json_value(v: &serde_yaml::Value) -> Result<serde_json::Value, String> {
        match v {
            serde_yaml::Value::Null => Ok(serde_json::Value::Null),

            serde_yaml::Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),

            serde_yaml::Value::Number(num) => {
                Ok(serde_json::Value::Number(yaml_number_as_json(num.clone())))
            }

            serde_yaml::Value::String(s) => Ok(serde_json::Value::String(s.clone())),

            serde_yaml::Value::Sequence(seq) => {
                let array_elements = seq
                    .into_iter()
                    .map(|v| as_json_value(v).expect("invalid sequence element"));
                Ok(serde_json::Value::Array(array_elements.collect()))
            }

            serde_yaml::Value::Mapping(map) => {
                let object_members = map.iter().map(|(k, v)| {
                    (
                        serde_yaml::to_string(k).expect("non-string mapping key"),
                        as_json_value(v).expect("invalid map value"),
                    )
                });
                Ok(serde_json::Value::Object(object_members.collect()))
            }
        }
    }

    fn as_json_value_array(v: &serde_yaml::Value) -> Result<Vec<serde_json::Value>, String> {
        match v {
            serde_yaml::Value::Sequence(seq) => {
                let array_elements = seq
                    .into_iter()
                    .map(|v| as_json_value(v).expect("invalid sequence element"));
                Ok(array_elements.collect())
            }
            _ => Err("not a sequence".to_string()),
        }
    }

    fn yaml_number_as_json(n: serde_yaml::Number) -> serde_json::Number {
        if n.is_i64() {
            serde_json::Number::from(n.as_i64().expect("invalid i64 in YAML"))
        } else if n.is_u64() {
            serde_json::Number::from(n.as_u64().expect("invalid u64 in YAML"))
        } else {
            serde_json::Number::from_f64(n.as_f64().expect("invalid f64 in YAML"))
                .expect("invalid f64 for JSON")
        }
    }
}
