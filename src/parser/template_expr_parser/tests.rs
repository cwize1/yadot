use std::{fs, path::Path};

use super::*;

#[test]
fn string_simple() {
    run_test("string_simple")
}

#[test]
fn string_with_whitespace() {
    run_test("string_with_whitespace")
}

#[test]
fn drop_simple() {
    run_test("drop_simple")
}

#[test]
fn drop_with_whitespace() {
    run_test("drop_with_whitespace")
}

#[test]
fn inline_simple() {
    run_test("inline_simple")
}

#[test]
fn inline_with_whitespace() {
    run_test("inline_with_whitespace")
}

fn run_test(name: &str) {
    let rootdir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_data_dir = rootdir.join("src/parser/template_expr_parser/tests/testdata");

    let test_file = test_data_dir.join(format!("tests/{}.txt", name));
    let expected_file = test_data_dir.join(format!("expected/{}.txt", name));
    let actual_dir = test_data_dir.join("actual");
    let actual_file = actual_dir.join(format!("{}.txt", name));

    let test = fs::read_to_string(test_file).unwrap();

    let parser = TemplateExprParser::new();
    let result = parser.parse(&test);
    let actual = format_result(result);

    fs::create_dir_all(actual_dir).unwrap();
    fs::write(actual_file, &actual).unwrap();

    let expected = fs::read_to_string(expected_file).unwrap();
    assert_eq!(expected, actual);
}

fn format_result(result: Result<(Expr, usize), Error>) -> String {
    let mut string = String::new();

    string.push_str("ERROR: ");

    if let Err(err) = result {
        string.push_str(&err.to_string());
        return string;
    }

    let (expr, end) = result.unwrap();

    string.push_str("<None>\n");
    string.push_str(&format!("END: {}\n", end));
    string.push_str("EXPR:\n");
    fomat_expr(&mut string, &expr);

    return string;
}

fn fomat_expr(string: &mut String, expr: &Expr) {
    match expr {
        Expr::String(value) => string.push_str(&format!("{:?}", value.value)),
        Expr::Inline => string.push_str("inline"),
        Expr::Drop => string.push_str("drop"),
    }
}
