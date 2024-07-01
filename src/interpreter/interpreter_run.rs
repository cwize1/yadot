use std::borrow::Cow;

use anyhow::{anyhow, Error};
use yaml_rust::{yaml::Hash, Yaml};

use crate::ast::{
    Expr, ExprString, FileTemplate, MapTemplate, NodeTemplate, ScalarTemplateValue, ScalerTemplate, SequenceTemplate,
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

enum ExprValue<'a> {
    String(ExprValueString<'a>),
    Inline,
    Drop,
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
            let value = self.expect_value(value)?;

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
            let value = self.expect_value(value)?;
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
                    let value = self.expect_value(value)?;
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
                    let value = self.expect_value(value)?;

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
        let mut string = String::new();
        let mut singular_value = None;

        for value_templ in &scalar_templ.values {
            match value_templ {
                ScalarTemplateValue::String(substring) => string.push_str(substring),
                ScalarTemplateValue::Expr(expr) => {
                    let expr_value = self.interpret_expr(expr)?;
                    match expr_value {
                        ExprValue::String(expr_string) => {
                            string.push_str(&expr_string.value);
                        }
                        ExprValue::Inline => {
                            singular_value = Some(Value::Inline);
                            break;
                        }
                        ExprValue::Drop => {
                            singular_value = Some(Value::Drop);
                            break;
                        }
                    }
                }
            }
        }

        match singular_value {
            Some(value) => {
                if scalar_templ.values.len() > 1 {
                    match value {
                        Value::Inline => return Err(anyhow!("expression value 'inline' cannot be a substring")),
                        Value::Drop => return Err(anyhow!("expression value 'drop' cannot be a substring")),
                        _ => unreachable!(),
                    }
                }

                Ok(value)
            }
            None => {
                let value = Value::Yaml(Yaml::String(string));
                Ok(value)
            }
        }
    }

    fn interpret_expr<'a>(&mut self, expr: &'a Expr) -> Result<ExprValue<'a>, Error> {
        match expr {
            Expr::String(expr_string) => self.interpret_expr_string(expr_string),
            Expr::Inline => self.interpret_expr_inline(),
            Expr::Drop => self.interpret_expr_drop(),
            Expr::Query(_) => todo!(),
        }
    }

    fn interpret_expr_string<'a>(&mut self, expr_string: &'a ExprString) -> Result<ExprValue<'a>, Error> {
        Ok(ExprValue::String(ExprValueString {
            value: Cow::Borrowed(&expr_string.value),
        }))
    }

    fn interpret_expr_inline<'a>(&mut self) -> Result<ExprValue<'a>, Error> {
        Ok(ExprValue::Inline)
    }

    fn interpret_expr_drop<'a>(&mut self) -> Result<ExprValue<'a>, Error> {
        Ok(ExprValue::Drop)
    }

    fn expect_value(&mut self, value: Value) -> Result<Value, Error> {
        match value {
            Value::Yaml(_) | Value::InlineYaml(_) | Value::Nothing => Ok(value),
            Value::Inline => Err(anyhow!("expression value 'inline' can only be used as a map key")),
            Value::Drop => Err(anyhow!("expression value 'drop' can only be used as a map key")),
        }
    }
}
