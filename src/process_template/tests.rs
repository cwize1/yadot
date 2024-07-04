// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use hashlink::LinkedHashMap;

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
    let tests = as_hash_mut(&mut tests_data_doc[&to_yaml_string("tests")]).unwrap();
    let test_data = as_hash_mut(&mut tests[&to_yaml_string(name)]).unwrap();

    let template = &test_data[&to_yaml_string("template")];
    let Yaml::String(template) = template else {
        panic!("test 'template' value should be a string")
    };
    let config = test_data.get(&to_yaml_string("config"));
    let config = match config {
        Some(Yaml::String(value)) => value.as_ref().as_str(),
        None => "",
        Some(_) => panic!("test 'config' value should be a string"),
    };

    let expected = test_data[&to_yaml_string("expected")].clone();

    let result = process_yaml_template(name, template, config);
    let actual = format_result(result);

    test_data.insert(to_yaml_string("expected"), actual.clone());
    yaml_emit_to_file(&tests_data_docs, &actual_data_file).unwrap();

    assert_eq!(expected, actual);
}

fn format_result(result: Result<Vec<Yaml>, Error>) -> Yaml {
    let (err, output) = match result {
        Ok(docs) => (Yaml::Null, Yaml::Array(Rc::new(docs))),
        Err(err) => (Yaml::String(Rc::new(format!("{:#}", err))), Yaml::Null),
    };

    let mut result = LinkedHashMap::new();
    result.insert(to_yaml_string("error"), err);
    result.insert(to_yaml_string("output"), output);

    let result = Yaml::Hash(Rc::new(result));
    result
}

fn as_hash_mut(yaml: &mut Yaml) -> Option<&mut LinkedHashMap<Yaml, Yaml>> {
    match yaml {
        Yaml::Hash(hash) => Some(Rc::get_mut(hash).unwrap()),
        _ => None,
    }
}

fn to_yaml_string(value: &str) -> Yaml {
    Yaml::String(Rc::new(value.to_string()))
}

fn yaml_emit_to_file(docs: &Vec<Yaml>, filename: &PathBuf) -> Result<(), Error> {
    let out = yaml_emit_to_string(docs)?;
    fs::write(filename, out)?;
    Ok(())
}

fn yaml_load_from_file(filename: &PathBuf) -> Result<Vec<Yaml>, Error> {
    let tests_data_str = fs::read_to_string(&filename)?;
    let tests_data_docs = parse_yaml_str(&tests_data_str)?;
    Ok(tests_data_docs)
}
