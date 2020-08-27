/*
 * Copyright 2020 VMware, Inc.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

use serde_json::Value;

pub enum FindError {
    // no errors yet
}

pub trait Path {
    fn find<'a>(&self, document: &'a Value) -> Result<Vec<&'a Value>, FindError>;
}
