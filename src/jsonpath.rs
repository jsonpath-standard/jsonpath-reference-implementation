/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

use serde_json::Value;

#[derive(Debug)]
pub enum SyntaxError {
    Message(String)
}

fn err(message: &str) -> Result<&dyn Path, SyntaxError> {
    Err(SyntaxError::Message(message.to_string()))
}

pub fn parse(_selector: &str) -> Result<&dyn Path, SyntaxError> {
    err("not implemented")
}

pub enum FindError {
    // no errors yet
}

pub trait Path {
    fn find(&self, document: Value) -> Result<Vec<Value>, FindError>;
}
