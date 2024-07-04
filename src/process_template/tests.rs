// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

use std::path::Path;

use yaml_rust::yaml::Hash;

use crate::yaml_utils::{yaml_emit_to_file, yaml_load_from_file};

use super::*;

macro_rules! testlist {
    ($($name:ident,)*) => {
    $(
        #[test]
        fn $name() {
            run_test(stringify!($name))
        }
    )*
    }
}

testlist! {
    drop_empty_field_value,
    drop_field,
    drop_simple,
    drop_substring,
    drop_value,
    if_config_bool,
    if_inline,
    if_drop,
    inline_double_drop,
    inline_double,
    inline_field,
    inline_list_into_list,
    inline_list_into_map,
    inline_map_into_list,
    inline_value_into_list,
    inline_value,
    boolean_as_substring,
    query_inline_object,
    query_simple,
    query_substring,
    query_not_found,
    query_index_wrong_type,
    query_index_string,
    query_index_bool,
    query_index_list,
    query_index_list_out_of_bounds,
    query_index_list_string,
    simple_list_expr,
    simple_list,
    inline_simple,
    inline_substring,
    inline_value_into_map,
    simple_string,
    simple_string_expr,
    true_value,
    false_value,
    true_eq_true,
    true_eq_false,
    true_ne_false,
    empty_string_eq,
    empty_string_ne,
    string_not_empty,
    string_eq_itself,
    string_eq_config_var,
    if_config_var,
}

fn run_test(name: &str) {
    let rootdir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let test_data_dir = rootdir.join("src/process_template/tests/testdata");
    let tests_data_file = test_data_dir.join("tests.yaml");
    let actual_data_file = test_data_dir.join("actual.yaml");

    let mut tests_data_docs = yaml_load_from_file(&tests_data_file).unwrap();
    let tests_data_doc = as_hash_mut(&mut tests_data_docs[0]).unwrap();
    let tests = as_hash_mut(&mut tests_data_doc[&Yaml::String("tests".to_string())]).unwrap();
    let test_data = as_hash_mut(&mut tests[&Yaml::String(name.to_string())]).unwrap();

    let template = test_data[&Yaml::String("template".to_string())].as_str().unwrap();
    let config = test_data.get(&Yaml::String("config".to_string()));
    let config = match config {
        Some(yaml) => yaml.as_str().unwrap(),
        None => "",
    };

    let expected = test_data[&Yaml::String("expected".to_string())].clone();

    let result = process_yaml_template(name, template, config);
    let actual = format_result(result);

    test_data.insert(Yaml::String("expected".to_string()), actual.clone());
    yaml_emit_to_file(&tests_data_docs, &actual_data_file).unwrap();

    assert_eq!(expected, actual);
}

fn format_result(result: Result<Vec<Yaml>, Error>) -> Yaml {
    let (err, output) = match result {
        Ok(docs) => (Yaml::Null, Yaml::Array(docs)),
        Err(err) => (Yaml::String(format!("{:#}", err)), Yaml::Null),
    };

    let mut result = Hash::new();
    result.insert(Yaml::String("error".to_string()), err);
    result.insert(Yaml::String("output".to_string()), output);

    let result = Yaml::Hash(result);
    result
}

fn as_hash_mut(yaml: &mut Yaml) -> Option<&mut Hash> {
    match yaml {
        Yaml::Hash(hash) => Some(hash),
        _ => None,
    }
}
