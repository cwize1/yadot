use std::borrow::Cow;

use anyhow::{anyhow, Error};
use yaml_rust::{yaml::Hash, Yaml};

use crate::ast::{
    Expr, ExprQuery, ExprString, FileTemplate, MapTemplate, NodeTemplate, ScalarTemplateValue, ScalerTemplate,
    SequenceTemplate, Statement, StatementIf,
};

pub struct InterpreterRun<'a> {
    config: &'a Yaml,
}

enum Value {
    Yaml(Yaml),
    InlineYaml(Yaml),
    Inline,
    Drop,
    Nothing,
}

enum ScalarValue<'a> {
    String(&'a str),
    Inline,
    Drop,
    Yaml(Yaml),
}

enum ExprValue {
    Inline,
    Drop,
    Yaml(Yaml),
}

struct ExprValueString<'a> {
    value: Cow<'a, str>,
}

impl InterpreterRun<'_> {
    pub fn new<'a>(config: &'a Yaml) -> InterpreterRun<'a> {
        InterpreterRun { config }
    }

    pub fn interpret_file(&mut self, file_templ: &FileTemplate) -> Result<Vec<Yaml>, Error> {
        let mut docs = Vec::new();
        for doc_templ in &file_templ.docs {
            let value = self.interpret_node(&doc_templ.node)?;
            let value = Self::expect_value(value)?;

            match value {
                Value::Yaml(value) | Value::InlineYaml(value) => {
                    docs.push(value);
                }
                Value::Nothing => {}
                Value::Inline | Value::Drop => unreachable!(),
            };
        }

        Ok(docs)
    }

    fn interpret_node(&mut self, node_templ: &NodeTemplate) -> Result<Value, Error> {
        match node_templ {
            NodeTemplate::Sequence(seq_templ) => self.interpret_seq(seq_templ),
            NodeTemplate::Map(map_templ) => self.interpret_map(map_templ),
            NodeTemplate::Scaler(scalar_templ) => self.interpret_scalar(scalar_templ),
        }
    }

    fn interpret_seq(&mut self, seq_templ: &SequenceTemplate) -> Result<Value, Error> {
        let mut values = Vec::new();
        for value_templ in &seq_templ.values {
            let value = self.interpret_node(value_templ)?;
            let value = Self::expect_value(value)?;
            match value {
                Value::Yaml(value) => {
                    values.push(value);
                }
                Value::InlineYaml(value) => {
                    let sublist = match value {
                        Yaml::Array(sublist) => sublist,
                        Yaml::Hash(_) => return Err(anyhow!("cannot inline maps into lists")),
                        Yaml::Real(_) | Yaml::Integer(_) | Yaml::String(_) | Yaml::Boolean(_) | Yaml::Null => {
                            return Err(anyhow!("cannot inline values into lists"))
                        }
                        Yaml::Alias(_) | Yaml::BadValue => unreachable!(),
                    };

                    // Merge sublist into this list.
                    for item in sublist {
                        values.push(item);
                    }
                }
                Value::Nothing => {}
                Value::Inline | Value::Drop => unreachable!(),
            }
        }

        let seq = Yaml::Array(values);
        let value = Value::Yaml(seq);
        Ok(value)
    }

    fn interpret_map(&mut self, map_templ: &MapTemplate) -> Result<Value, Error> {
        let mut entries = Hash::new();
        for entry_templ in &map_templ.entries {
            let key = self.interpret_node(&entry_templ.key)?;
            match key {
                Value::Yaml(key) => {
                    let value = self.interpret_node(&entry_templ.value)?;
                    let value = Self::expect_value(value)?;
                    let value = match value {
                        Value::Yaml(value) | Value::InlineYaml(value) => value,
                        // In YAML, a key without a value is given a default value of null.
                        Value::Nothing => Yaml::Null,
                        Value::Inline | Value::Drop => unreachable!(),
                    };
                    entries.insert(key, value);
                }
                Value::Inline => {
                    let value = self.interpret_node(&entry_templ.value)?;
                    let value = Self::expect_value(value)?;

                    // Check if the only item in the map is the inline expression.
                    if map_templ.entries.len() == 1 {
                        let value = match value {
                            // Report value as inlined.
                            Value::Yaml(value) => Value::InlineYaml(value),
                            _ => value,
                        };
                        return Ok(value);
                    }

                    match value {
                        Value::Yaml(value) | Value::InlineYaml(value) => match value {
                            Yaml::Hash(submap) => {
                                for (key, value) in submap {
                                    entries.insert(key, value);
                                }
                            }
                            Yaml::Array(_) => return Err(anyhow!("cannot inline lists into maps")),
                            Yaml::Real(_) | Yaml::Integer(_) | Yaml::String(_) | Yaml::Boolean(_) | Yaml::Null => {
                                return Err(anyhow!("cannot inline values into maps"))
                            }
                            Yaml::Alias(_) | Yaml::BadValue => unreachable!(),
                        },
                        Value::Nothing => {}
                        Value::Inline | Value::Drop => unreachable!(),
                    }
                }
                Value::Drop => {
                    // Check if the only item in the map is the inline expression.
                    if map_templ.entries.len() == 1 {
                        // Return nothing, to remove the map.
                        let value = Value::Nothing;
                        return Ok(value);
                    }
                }
                Value::Nothing => {}
                Value::InlineYaml(_) => unreachable!(),
            }
        }

        let map = Yaml::Hash(entries);
        let value = Value::Yaml(map);
        Ok(value)
    }

    fn interpret_scalar(&mut self, scalar_templ: &ScalerTemplate) -> Result<Value, Error> {
        let mut values = Vec::new();
        for value_templ in &scalar_templ.values {
            match value_templ {
                ScalarTemplateValue::String(substring) => {
                    values.push(ScalarValue::String(substring));
                }
                ScalarTemplateValue::Expr(stmt) => {
                    let value = self.interpret_statement(stmt)?;
                    let value = match value {
                        ExprValue::Inline => ScalarValue::Inline,
                        ExprValue::Drop => ScalarValue::Drop,
                        ExprValue::Yaml(yaml) => ScalarValue::Yaml(yaml),
                    };
                    values.push(value);
                }
            }
        }

        if values.len() == 1 {
            let singular_value = &values[0];
            let value = match singular_value {
                ScalarValue::String(string) => Value::Yaml(Yaml::String(string.to_string())),
                ScalarValue::Inline => Value::Inline,
                ScalarValue::Drop => Value::Drop,
                ScalarValue::Yaml(yaml) => Value::Yaml(yaml.clone()),
            };
            return Ok(value);
        }

        let mut string = String::new();
        for value in values {
            match value {
                ScalarValue::String(expr_string) => {
                    string.push_str(expr_string);
                }
                ScalarValue::Inline => return Err(anyhow!("expression value 'inline' cannot be a substring")),
                ScalarValue::Drop => return Err(anyhow!("expression value 'drop' cannot be a substring")),
                ScalarValue::Yaml(yaml) => match yaml {
                    Yaml::String(substring) => {
                        string.push_str(&substring);
                    }
                    _ => return Err(anyhow!("expression value cannot be a substring: value is not a string")),
                },
            }
        }
        let string_value = Value::Yaml(Yaml::String(string));
        Ok(string_value)
    }

    fn interpret_statement(&mut self, stmt: &Statement) -> Result<ExprValue, Error> {
        match stmt {
            Statement::Expr(expr) => self.interpret_expr(expr),
            Statement::If(if_stmt) => self.interpret_if(if_stmt),
        }
    }

    fn interpret_if(&mut self, if_stmt: &StatementIf) -> Result<ExprValue, Error> {
        let conditional = self.interpret_expr(&if_stmt.condition)?;
        let conditional = Self::expect_implicit_bool(conditional)?;
        match conditional {
            true => Ok(ExprValue::Inline),
            false => Ok(ExprValue::Drop),
        }
    }

    fn interpret_expr(&mut self, expr: &Expr) -> Result<ExprValue, Error> {
        match expr {
            Expr::String(expr_string) => self.interpret_string(expr_string),
            Expr::Inline => self.interpret_inline(),
            Expr::Drop => self.interpret_drop(),
            Expr::Query(query) => self.interpret_query(query),
        }
    }

    fn interpret_string(&mut self, expr_string: &ExprString) -> Result<ExprValue, Error> {
        Ok(ExprValue::Yaml(Yaml::String(expr_string.value.clone())))
    }

    fn interpret_inline(&mut self) -> Result<ExprValue, Error> {
        Ok(ExprValue::Inline)
    }

    fn interpret_drop(&mut self) -> Result<ExprValue, Error> {
        Ok(ExprValue::Drop)
    }

    fn interpret_query(&mut self, query: &ExprQuery) -> Result<ExprValue, Error> {
        let value = self.query(query)?;
        Ok(ExprValue::Yaml(value.clone()))
    }

    fn query(&mut self, query: &ExprQuery) -> Result<&Yaml, Error> {
        match query {
            ExprQuery::Root => Ok(&self.config),
            ExprQuery::ObjectIndex(objectindex) => {
                let object = self.query(&objectindex.object)?;
                match object {
                    Yaml::Hash(object) => {
                        let subvalue = object.get(&Yaml::String(objectindex.index.name.clone()));
                        match subvalue {
                            Some(subvalue) => Ok(subvalue),
                            None => Err(anyhow!("index '{}' value not found", objectindex.index.name)),
                        }
                    }
                    _ => Err(anyhow!(
                        "cannot get index '{}': value type is not indexable",
                        objectindex.index.name
                    )),
                }
            }
        }
    }

    fn expect_value(value: Value) -> Result<Value, Error> {
        match value {
            Value::Yaml(_) | Value::InlineYaml(_) | Value::Nothing => Ok(value),
            Value::Inline => Err(anyhow!("expression value 'inline' can only be used as a map key")),
            Value::Drop => Err(anyhow!("expression value 'drop' can only be used as a map key")),
        }
    }

    fn expect_implicit_bool(value: ExprValue) -> Result<bool, Error> {
        // Convrert null and false to false. All other valid values are true.
        // This matches jq's semantics.
        match value {
            ExprValue::Inline => Err(anyhow!("expression value 'inline' cannot be converted to a bool value")),
            ExprValue::Drop => Err(anyhow!("expression value 'drop' cannot be converted to a bool value")),
            ExprValue::Yaml(Yaml::BadValue) => unreachable!(),
            ExprValue::Yaml(Yaml::Boolean(false)) | ExprValue::Yaml(Yaml::Null) => Ok(false),
            ExprValue::Yaml(_) => Ok(true),
        }
    }
}
