// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

// Provides copy-on-write variant of a YAML object.

mod loader;

use std::rc::Rc;

use hashlink::LinkedHashMap;

pub use loader::parse_yaml_str;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum Yaml {
    // Numbers that don't fit in an i64 (e.g. floating point).
    Real(Rc<String>),
    Integer(i64),
    String(Rc<String>),
    Boolean(bool),
    Array(Rc<Vec<Yaml>>),
    Hash(Rc<LinkedHashMap<Yaml, Yaml>>),
    Null,
}
