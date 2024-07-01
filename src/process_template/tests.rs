use std::{fs, io, path::Path};

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
    inline_double_drop,
    inline_double,
    inline_field,
    inline_list_into_list,
    inline_list_into_map,
    inline_value,
    query_inline_object,
    query_simple,
    query_substring,
    simple_list_expr,
    simple_list,
    inline_simple,
    inline_substring,
    inline_value_into_map,
    simple_string,
    simple_string_expr,
}

fn run_test(name: &str) {
    let rootdir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_data_dir = rootdir.join("src/process_template/tests/testdata");

    let test_file = test_data_dir.join(format!("tests/{}.txt", name));
    let test_config_file = test_data_dir.join(format!("tests/{}-config.txt", name));
    let expected_file = test_data_dir.join(format!("expected/{}.txt", name));
    let actual_dir = test_data_dir.join("actual");
    let actual_file = actual_dir.join(format!("{}.txt", name));

    let test = fs::read_to_string(&test_file).unwrap();
    let test_config = fs::read_to_string(&test_config_file);
    let test_config = match test_config {
        Ok(test_config) => test_config,
        Err(err) if err.kind() == io::ErrorKind::NotFound => "".to_string(),
        _ => test_config.unwrap(),
    };

    let result = process_yaml_template(&test, &test_config);
    let actual = format_result(result);

    fs::create_dir_all(actual_dir).unwrap();
    fs::write(actual_file, &actual).unwrap();

    let expected = fs::read_to_string(expected_file).unwrap();
    assert_eq!(expected, actual);
}

fn format_result(result: Result<String, Error>) -> String {
    let mut string = String::new();

    string.push_str("ERROR: ");
    if let Err(err) = result {
        string.push_str(&err.to_string());
        return string;
    }

    let output = result.unwrap();

    string.push_str("<None>\n");
    string.push_str("OUTPUT:\n");
    string.push_str(&output);

    return string;
}
