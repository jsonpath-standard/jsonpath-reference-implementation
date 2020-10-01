/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

use crate::parser;
use crate::path::Path;

#[derive(Debug)]
pub struct SyntaxError {
    message: String,
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.message)
    }
}

pub fn parse(selector: &str) -> Result<Box<dyn Path + '_>, SyntaxError> {
    parser::parse(selector).map_err(|m| SyntaxError { message: m })
}
