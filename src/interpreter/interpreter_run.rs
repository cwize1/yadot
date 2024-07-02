use anyhow::{anyhow, Error};
use yaml_rust::{yaml::Hash, Yaml};

use crate::ast::{
    Expr, ExprQuery, ExprString, FileTemplate, MapTemplate, NodeTemplate, ScalarTemplateValue, ScalerTemplate,
    SequenceTemplate, SourceLocationSpan, Statement, StatementIf,
};

pub struct InterpreterRun<'a> {
    config: &'a Yaml,
}

struct Value {
    pub src_loc: SourceLocationSpan,
    pub data: ValueData,
}

enum ValueData {
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

macro_rules! errwithloc {
    ($loc:expr, $fmt:expr $(, $($arg:tt)*)?) => {
        anyhow!(concat!("{}:{}:{} ", $fmt), $loc.filename, $loc.start.line, $loc.start.col, $($($arg)*)?)
    };
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

            match value.data {
                ValueData::Yaml(value) | ValueData::InlineYaml(value) => {
                    docs.push(value);
                }
                ValueData::Nothing => {}
                ValueData::Inline | ValueData::Drop => unreachable!(),
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
            match value.data {
                ValueData::Yaml(yaml) => {
                    values.push(yaml);
                }
                ValueData::InlineYaml(yaml) => {
                    let sublist = match yaml {
                        Yaml::Array(sublist) => sublist,
                        Yaml::Hash(_) => return Err(errwithloc!(value.src_loc, "cannot inline maps into lists")),
                        Yaml::Real(_) | Yaml::Integer(_) | Yaml::String(_) | Yaml::Boolean(_) | Yaml::Null => {
                            return Err(errwithloc!(value.src_loc, "cannot inline values into lists"))
                        }
                        Yaml::Alias(_) | Yaml::BadValue => unreachable!(),
                    };

                    // Merge sublist into this list.
                    for item in sublist {
                        values.push(item);
                    }
                }
                ValueData::Nothing => {}
                ValueData::Inline | ValueData::Drop => unreachable!(),
            }
        }

        let seq = Yaml::Array(values);
        let data = ValueData::Yaml(seq);
        let value = Value {
            src_loc: seq_templ.src_loc.clone(),
            data,
        };
        Ok(value)
    }

    fn interpret_map(&mut self, map_templ: &MapTemplate) -> Result<Value, Error> {
        let mut entries = Hash::new();
        for entry_templ in &map_templ.entries {
            let key = self.interpret_node(&entry_templ.key)?;
            match key.data {
                ValueData::Yaml(key) => {
                    let value = self.interpret_node(&entry_templ.value)?;
                    let value = Self::expect_value(value)?;
                    let value = match value.data {
                        ValueData::Yaml(value) | ValueData::InlineYaml(value) => value,
                        // In YAML, a key without a value is given a default value of null.
                        ValueData::Nothing => Yaml::Null,
                        ValueData::Inline | ValueData::Drop => unreachable!(),
                    };
                    entries.insert(key, value);
                }
                ValueData::Inline => {
                    let value = self.interpret_node(&entry_templ.value)?;
                    let value = Self::expect_value(value)?;

                    // Check if the only item in the map is the inline expression.
                    if map_templ.entries.len() == 1 {
                        let data = match value.data {
                            // Report value as inlined.
                            ValueData::Yaml(yaml) => ValueData::InlineYaml(yaml),
                            _ => value.data,
                        };
                        let value = Value {
                            src_loc: value.src_loc,
                            data,
                        };
                        return Ok(value);
                    }

                    match value.data {
                        ValueData::Yaml(yaml) | ValueData::InlineYaml(yaml) => match yaml {
                            Yaml::Hash(submap) => {
                                for (key, value) in submap {
                                    entries.insert(key, value);
                                }
                            }
                            Yaml::Array(_) => return Err(errwithloc!(value.src_loc, "cannot inline lists into maps")),
                            Yaml::Real(_) | Yaml::Integer(_) | Yaml::String(_) | Yaml::Boolean(_) | Yaml::Null => {
                                return Err(errwithloc!(value.src_loc, "cannot inline values into maps"))
                            }
                            Yaml::Alias(_) | Yaml::BadValue => unreachable!(),
                        },
                        ValueData::Nothing => {}
                        ValueData::Inline | ValueData::Drop => unreachable!(),
                    }
                }
                ValueData::Drop => {
                    // Check if the only item in the map is the inline expression.
                    if map_templ.entries.len() == 1 {
                        // Return nothing, to remove the map.
                        let data = ValueData::Nothing;
                        let value = Value {
                            src_loc: key.src_loc,
                            data,
                        };
                        return Ok(value);
                    }
                }
                ValueData::Nothing => {}
                ValueData::InlineYaml(_) => unreachable!(),
            }
        }

        let map = Yaml::Hash(entries);
        let data = ValueData::Yaml(map);
        let value = Value {
            src_loc: map_templ.src_loc.clone(),
            data,
        };
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
                    let value = self.interpret_statement(stmt, &scalar_templ.src_loc)?;
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
            let data = match singular_value {
                ScalarValue::String(string) => ValueData::Yaml(Yaml::String(string.to_string())),
                ScalarValue::Inline => ValueData::Inline,
                ScalarValue::Drop => ValueData::Drop,
                ScalarValue::Yaml(yaml) => ValueData::Yaml(yaml.clone()),
            };
            let value = Value {
                src_loc: scalar_templ.src_loc.clone(),
                data,
            };
            return Ok(value);
        }

        let mut string = String::new();
        for value in values {
            match value {
                ScalarValue::String(expr_string) => {
                    string.push_str(expr_string);
                }
                ScalarValue::Inline => {
                    return Err(errwithloc!(
                        scalar_templ.src_loc,
                        "expression value 'inline' cannot be a substring"
                    ))
                }
                ScalarValue::Drop => {
                    return Err(errwithloc!(
                        scalar_templ.src_loc,
                        "expression value 'drop' cannot be a substring"
                    ))
                }
                ScalarValue::Yaml(yaml) => match yaml {
                    Yaml::String(substring) => {
                        string.push_str(&substring);
                    }
                    _ => {
                        return Err(errwithloc!(
                            scalar_templ.src_loc,
                            "expression value cannot be a substring: value is not a string"
                        ))
                    }
                },
            }
        }
        let data = ValueData::Yaml(Yaml::String(string));
        let value = Value {
            src_loc: scalar_templ.src_loc.clone(),
            data,
        };
        Ok(value)
    }

    fn interpret_statement(&mut self, stmt: &Statement, src_loc: &SourceLocationSpan) -> Result<ExprValue, Error> {
        match stmt {
            Statement::Expr(expr) => self.interpret_expr(expr, src_loc),
            Statement::If(if_stmt) => self.interpret_if(if_stmt, src_loc),
        }
    }

    fn interpret_if(&mut self, if_stmt: &StatementIf, src_loc: &SourceLocationSpan) -> Result<ExprValue, Error> {
        let conditional = self.interpret_expr(&if_stmt.condition, src_loc)?;
        let conditional = Self::expect_implicit_bool(conditional, src_loc)?;
        match conditional {
            true => Ok(ExprValue::Inline),
            false => Ok(ExprValue::Drop),
        }
    }

    fn interpret_expr(&mut self, expr: &Expr, src_loc: &SourceLocationSpan) -> Result<ExprValue, Error> {
        match expr {
            Expr::String(expr_string) => self.interpret_string(expr_string),
            Expr::Inline => self.interpret_inline(),
            Expr::Drop => self.interpret_drop(),
            Expr::Query(query) => self.interpret_query(query, src_loc),
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

    fn interpret_query(&mut self, query: &ExprQuery, src_loc: &SourceLocationSpan) -> Result<ExprValue, Error> {
        let value = self.query(query, src_loc)?;
        Ok(ExprValue::Yaml(value.clone()))
    }

    fn query(&mut self, query: &ExprQuery, src_loc: &SourceLocationSpan) -> Result<&Yaml, Error> {
        match query {
            ExprQuery::Root => Ok(&self.config),
            ExprQuery::ObjectIndex(objectindex) => {
                let object = self.query(&objectindex.object, src_loc)?;
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
        match value.data {
            ValueData::Yaml(_) | ValueData::InlineYaml(_) | ValueData::Nothing => Ok(value),
            ValueData::Inline => Err(errwithloc!(
                value.src_loc,
                "expression value 'inline' can only be used as a map key"
            )),
            ValueData::Drop => Err(errwithloc!(
                value.src_loc,
                "expression value 'drop' can only be used as a map key"
            )),
        }
    }

    fn expect_implicit_bool(value: ExprValue, src_loc: &SourceLocationSpan) -> Result<bool, Error> {
        // Convrert null and false to false. All other valid values are true.
        // This matches jq's semantics.
        match value {
            ExprValue::Inline => Err(errwithloc!(src_loc, "expression value 'inline' cannot be converted to a bool value")),
            ExprValue::Drop => Err(errwithloc!(src_loc, "expression value 'drop' cannot be converted to a bool value")),
            ExprValue::Yaml(Yaml::BadValue) => unreachable!(),
            ExprValue::Yaml(Yaml::Boolean(false)) | ExprValue::Yaml(Yaml::Null) => Ok(false),
            ExprValue::Yaml(_) => Ok(true),
        }
    }
}
