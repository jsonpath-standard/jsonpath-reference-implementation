/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

use crate::ast;
use crate::parser;
use serde_json::Value;

#[derive(Debug)]
pub struct SyntaxError {
    message: String,
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

pub enum FindError {
    // no errors yet
}

pub fn parse(selector: &str) -> Result<Path, SyntaxError> {
    let p = parser::parse(selector).map_err(|m| SyntaxError { message: m })?;
    Ok(Path(p))
}

pub struct Path(ast::Path);

impl Path {
    pub fn find<'a>(&'a self, document: &'a Value) -> Result<Vec<&'a Value>, FindError> {
        Ok(self.0.find(document).collect())
    }
}
