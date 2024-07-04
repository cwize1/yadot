// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

use std::rc::Rc;

use anyhow::Error;
use hashlink::LinkedHashMap;
use saphyr::YamlEmitter;

use crate::cow_yaml::Yaml;

pub fn yaml_emit_to_string(docs: &Vec<Yaml>) -> Result<String, Error> {
    let docs = docs_to_yaml_rust_type(docs);

    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    for doc in docs {
        emitter.dump(&doc)?;
    }
    Ok(out_str)
}

pub fn docs_to_yaml_rust_type(docs: &Vec<Yaml>) -> Vec<saphyr::Yaml> {
    let mut res = Vec::new();
    for doc in docs {
        let doc_res = to_yaml_rust_type(doc);
        res.push(doc_res);
    }
    res
}

pub fn to_yaml_rust_type(docs: &Yaml) -> saphyr::Yaml {
    match docs {
        Yaml::Real(value) => saphyr::Yaml::Real(value.as_ref().clone()),
        Yaml::Integer(value) => saphyr::Yaml::Integer(*value),
        Yaml::String(value) => saphyr::Yaml::Real(value.as_ref().clone()),
        Yaml::Boolean(value) => saphyr::Yaml::Boolean(*value),
        Yaml::Array(value) => list_to_yaml_rust_type(value),
        Yaml::Hash(value) => map_to_yaml_rust_type(value),
        Yaml::Null => saphyr::Yaml::Null,
    }
}

fn list_to_yaml_rust_type(list: &Rc<Vec<Yaml>>) -> saphyr::Yaml {
    let mut res = Vec::new();
    for node in list.as_ref() {
        let node_res = to_yaml_rust_type(node);
        res.push(node_res);
    }
    let res = saphyr::Yaml::Array(res);
    res
}

fn map_to_yaml_rust_type(list: &Rc<LinkedHashMap<Yaml, Yaml>>) -> saphyr::Yaml {
    let mut res = LinkedHashMap::new();
    for (key, value) in list.as_ref() {
        let key_res = to_yaml_rust_type(key);
        let value_res = to_yaml_rust_type(value);
        res.insert(key_res, value_res);
    }
    let res = saphyr::Yaml::Hash(res);
    res
}
