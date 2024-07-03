use std::{fs, path::Path};

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
    drop_simple,
    drop_with_whitespace,
    inline_simple,
    inline_with_whitespace,
    query_child,
    query_nested_child,
    query_root,
    string_simple,
    string_with_whitespace,
}

fn run_test(name: &str) {
    let rootdir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_data_dir = rootdir.join("src/parser/template_expr/parser/tests/testdata");

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

fn format_result(result: Result<(Statement, usize), Error>) -> String {
    let mut string = String::new();

    string.push_str("ERROR: ");

    if let Err(err) = result {
        string.push_str(&err.to_string());
        return string;
    }

    let (statement, end) = result.unwrap();

    string.push_str("<None>\n");
    string.push_str(&format!("END: {}\n", end));
    string.push_str("OUT:\n");
    fomat_statement(&mut string, &statement);

    return string;
}

fn fomat_statement(string: &mut String, statement: &Statement) {
    match statement {
        Statement::Expr(expr) => fomat_expr(string, expr),
        Statement::If(statement) => fomat_if(string, statement),
    }
}

fn fomat_if(string: &mut String, statement: &StatementIf) {
    string.push_str("if (");
    fomat_expr(string, &statement.condition);
    string.push_str("if )");
}

fn fomat_expr(string: &mut String, expr: &Expr) {
    match expr {
        Expr::String(value) => string.push_str(&format!("{:?}", value.value)),
        Expr::Inline => string.push_str("inline"),
        Expr::Drop => string.push_str("drop"),
        Expr::Query(query) => fomat_expr_query(string, query),
        Expr::True => string.push_str("true"),
        Expr::False => string.push_str("false"),
        Expr::Eq(_) => todo!(),
        Expr::Ne(_) => todo!(),
    }
}

fn fomat_expr_query(string: &mut String, query: &ExprQuery) {
    match query {
        ExprQuery::Root => string.push_str("."),
        ExprQuery::Index(ExprIndex { object, index }) => {
            string.push_str("(");
            fomat_expr_query(string, object);
            string.push_str(").");
            string.push_str("[");
            fomat_expr(string, index);
            string.push_str("]");
        }
    }
}

fn fomat_binary_op(string: &mut String, op: &str, binary_op: &ExprOpBinary) {
    string.push_str("(");
    fomat_expr(string, &binary_op.left);
    string.push_str(")");
    string.push_str(op);
    string.push_str("(");
    fomat_expr(string, &binary_op.right);
    string.push_str(")");
}
