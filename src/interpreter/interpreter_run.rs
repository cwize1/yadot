// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

use std::{collections::HashMap, rc::Rc};

use anyhow::{anyhow, Error};
use hashlink::LinkedHashMap;

use crate::{
    ast::{
        Expr, ExprBinding, ExprIndex, ExprInteger, ExprOpBinary, ExprQuery, ExprReal, ExprString, FileTemplate,
        MapTemplate, NodeTemplate, ScalarTemplateValue, ScalerTemplate, SequenceTemplate, SourceLocationSpan,
        Statement, StatementFor, StatementIf,
    },
    cow_yaml::Yaml,
};

pub struct InterpreterRun {
    config: Yaml,
    scopes: Vec<Scope>,
}

struct Scope {
    pub variables: HashMap<String, Yaml>,
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
    For(ValueFor),
}

enum ScalarValue {
    Inline,
    Drop,
    Yaml(Yaml),
    For(ValueFor),
}

struct ValueFor {
    pub bindings: Vec<ExprBinding>,
    pub iterable: Yaml,
}

#[derive(Clone, Debug, PartialEq)]
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

impl InterpreterRun {
    pub fn new(config: Yaml, variables: HashMap<String, Yaml>) -> InterpreterRun {
        InterpreterRun {
            config,
            scopes: vec![Scope { variables }],
        }
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
                ValueData::Inline | ValueData::Drop | ValueData::For(..) => unreachable!(),
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
                    };

                    // Merge sublist into this list.
                    for item in sublist.as_ref() {
                        values.push(item.clone());
                    }
                }
                ValueData::Nothing => {}
                // Checked by expect_value()
                ValueData::Inline | ValueData::Drop | ValueData::For(_) => unreachable!(),
            }
        }

        let seq = Yaml::Array(Rc::new(values));
        let data = ValueData::Yaml(seq);
        let value = Value {
            src_loc: seq_templ.src_loc.clone(),
            data,
        };
        Ok(value)
    }

    fn interpret_map(&mut self, map_templ: &MapTemplate) -> Result<Value, Error> {
        // If there is only a single item in a map and that item is a template expression,
        // then we allow inline and drop commands to apply to the parent value.
        let one_item_map = map_templ.entries.len() == 1;

        let mut entries = LinkedHashMap::new();
        for entry_templ in &map_templ.entries {
            let key = self.interpret_node(&entry_templ.key)?;
            match key.data {
                ValueData::Yaml(key) => {
                    let entry_value = self.interpret_node(&entry_templ.value)?;
                    let entry_value = Self::expect_value(entry_value)?;
                    let entry_value = match entry_value.data {
                        ValueData::Yaml(yaml) | ValueData::InlineYaml(yaml) => yaml,
                        // In YAML, a key without a value is given a default value of null.
                        ValueData::Nothing => Yaml::Null,
                        // Checked by expect_value().
                        ValueData::Inline | ValueData::Drop | ValueData::For(_) => unreachable!(),
                    };
                    entries.insert(key, entry_value);
                }
                key_data @ ValueData::Inline | key_data @ ValueData::For(_) => {
                    let entry_value = match key_data {
                        ValueData::Inline => self.interpret_node(&entry_templ.value)?,
                        ValueData::For(value_for) => self.run_for_loop(&key.src_loc, value_for, &entry_templ.value)?,
                        _ => unreachable!(),
                    };

                    let entry_value = Self::expect_value(entry_value)?;

                    // Check if the only item in the map is the inline expression.
                    if one_item_map {
                        let data = match entry_value.data {
                            // Report value as inlined.
                            ValueData::Yaml(yaml) => ValueData::InlineYaml(yaml),
                            _ => entry_value.data,
                        };
                        let value = Value {
                            src_loc: entry_value.src_loc,
                            data,
                        };
                        return Ok(value);
                    }

                    match entry_value.data {
                        ValueData::Yaml(yaml) | ValueData::InlineYaml(yaml) => match yaml {
                            // Pull up the lower map's entries into this map.
                            Yaml::Hash(submap) => {
                                for (key, value) in Rc::unwrap_or_clone(submap) {
                                    entries.insert(key, value);
                                }
                            }
                            Yaml::Array(_) => {
                                return Err(errwithloc!(entry_value.src_loc, "cannot inline lists into maps"))
                            }
                            Yaml::Real(_) | Yaml::Integer(_) | Yaml::String(_) | Yaml::Boolean(_) | Yaml::Null => {
                                return Err(errwithloc!(entry_value.src_loc, "cannot inline values into maps"))
                            }
                        },
                        ValueData::Nothing => {}
                        // Checked by expect_value().
                        ValueData::Inline | ValueData::Drop | ValueData::For(_) => unreachable!(),
                    }
                }
                ValueData::Drop => {
                    // Check if the only item in the map is the inline expression.
                    if one_item_map {
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

        let map = Yaml::Hash(Rc::new(entries));
        let data = ValueData::Yaml(map);
        let value = Value {
            src_loc: map_templ.src_loc.clone(),
            data,
        };
        Ok(value)
    }

    fn run_for_loop(
        &mut self,
        key_src_loc: &SourceLocationSpan,
        value_for: ValueFor,
        item_templ: &NodeTemplate,
    ) -> Result<Value, Error> {
        let mut combined_list = None;
        let mut combined_map = None;

        let mut handle_item = |item| -> Result<(), Error> {
            let item = Self::expect_value(item)?;
            match item.data {
                ValueData::Yaml(yaml) | ValueData::InlineYaml(yaml) => match yaml {
                    Yaml::Array(lower_list) => {
                        if let Some(_) = combined_map {
                            return Err(errwithloc!(item.src_loc, "cannot combine list item into map"));
                        }

                        if let None = combined_list {
                            combined_list = Some(Vec::new());
                        }

                        let combined_list = combined_list.as_mut().unwrap();

                        for item in lower_list.as_ref() {
                            combined_list.push(item.clone());
                        }
                    }
                    Yaml::Hash(lower_map) => {
                        if let Some(_) = combined_list {
                            return Err(errwithloc!(item.src_loc, "cannot combine map item into list"));
                        }

                        if let None = combined_map {
                            combined_map = Some(LinkedHashMap::new());
                        }

                        let combined_map = combined_map.as_mut().unwrap();

                        for (key, value) in lower_map.as_ref() {
                            combined_map.insert(key.clone(), value.clone());
                        }
                    }
                    _ => {
                        return Err(errwithloc!(
                            item.src_loc,
                            "for loop child item must be either an array or a map"
                        ))
                    }
                },
                ValueData::Nothing => {}
                // Checked by expect_value().
                _ => unreachable!(),
            }

            Ok(())
        };

        match value_for.iterable {
            Yaml::Array(list) => {
                if value_for.bindings.len() != 1 {
                    return Err(errwithloc!(
                        key_src_loc,
                        "for loop over list requires 1 binding item, found {}",
                        value_for.bindings.len()
                    ));
                }

                for item in list.as_ref() {
                    self.push_scope();
                    self.add_binding(&value_for.bindings[0], item);

                    let item = self.interpret_node(item_templ)?;

                    self.pop_scope();

                    handle_item(item)?;
                }
            }
            Yaml::Hash(map) => {
                if value_for.bindings.len() != 2 {
                    return Err(errwithloc!(
                        key_src_loc,
                        "for loop over map requires 2 binding items, found {}",
                        value_for.bindings.len()
                    ));
                }

                for (key, value) in map.as_ref() {
                    self.push_scope();

                    self.add_binding(&value_for.bindings[0], key);
                    self.add_binding(&value_for.bindings[1], value);

                    let item = self.interpret_node(item_templ)?;

                    self.pop_scope();

                    handle_item(item)?;
                }
            }
            // Check by expect_iterable() in interpret_for().
            _ => unreachable!(),
        }

        let mut data = ValueData::Nothing;

        if let Some(combined_list) = combined_list {
            let list = Yaml::Array(Rc::new(combined_list));
            data = ValueData::Yaml(list);
        }

        if let Some(combined_map) = combined_map {
            let map = Yaml::Hash(Rc::new(combined_map));
            data = ValueData::Yaml(map);
        }

        let value = Value {
            src_loc: key_src_loc.clone(),
            data,
        };
        Ok(value)
    }

    fn interpret_scalar(&mut self, scalar_templ: &ScalerTemplate) -> Result<Value, Error> {
        let mut values = Vec::new();
        for value_templ in &scalar_templ.values {
            match value_templ {
                ScalarTemplateValue::String(substring) => {
                    values.push(ScalarValue::Yaml(Yaml::String(substring.clone())));
                }
                ScalarTemplateValue::Expr(stmt) => {
                    let value = self.interpret_statement(stmt, &scalar_templ.src_loc)?;
                    values.push(value);
                }
            }
        }

        if values.len() == 1 {
            let singular_value = values.into_iter().nth(0).unwrap();
            let data = match singular_value {
                ScalarValue::Inline => ValueData::Inline,
                ScalarValue::Drop => ValueData::Drop,
                ScalarValue::Yaml(yaml) => ValueData::Yaml(yaml),
                ScalarValue::For(value_for) => ValueData::For(value_for),
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
                            "expression value of type {} cannot be a substring",
                            Self::yaml_type_name(&yaml)
                        ))
                    }
                },
                ScalarValue::For(_) => {
                    return Err(errwithloc!(
                        scalar_templ.src_loc,
                        "expression value 'for' cannot be a substring"
                    ))
                }
            }
        }
        let data = ValueData::Yaml(Yaml::String(Rc::new(string)));
        let value = Value {
            src_loc: scalar_templ.src_loc.clone(),
            data,
        };
        Ok(value)
    }

    fn interpret_statement(&mut self, stmt: &Statement, src_loc: &SourceLocationSpan) -> Result<ScalarValue, Error> {
        match stmt {
            Statement::Expr(expr) => {
                let expr_value = self.interpret_expr(expr, src_loc)?;
                let scalar_value = match expr_value {
                    ExprValue::Inline => ScalarValue::Inline,
                    ExprValue::Drop => ScalarValue::Drop,
                    ExprValue::Yaml(yaml) => ScalarValue::Yaml(yaml),
                };
                Ok(scalar_value)
            }
            Statement::If(if_stmt) => self.interpret_if(if_stmt, src_loc),
            Statement::For(for_stmt) => self.interpret_for(for_stmt, src_loc),
        }
    }

    fn interpret_if(&mut self, if_stmt: &StatementIf, src_loc: &SourceLocationSpan) -> Result<ScalarValue, Error> {
        let conditional = self.interpret_expr(&if_stmt.condition, src_loc)?;
        let conditional = Self::expect_implicit_bool(conditional, src_loc)?;
        match conditional {
            true => Ok(ScalarValue::Inline),
            false => Ok(ScalarValue::Drop),
        }
    }

    fn interpret_for(&mut self, for_stmt: &StatementFor, src_loc: &SourceLocationSpan) -> Result<ScalarValue, Error> {
        let iterable = self.interpret_expr(&for_stmt.iterable, src_loc)?;
        let iterable = Self::expect_iterable(iterable, src_loc)?;
        let value_for = ValueFor {
            bindings: for_stmt.bindings.clone(),
            iterable,
        };
        let scalar_value = ScalarValue::For(value_for);
        Ok(scalar_value)
    }

    fn interpret_expr(&mut self, expr: &Expr, src_loc: &SourceLocationSpan) -> Result<ExprValue, Error> {
        match expr {
            Expr::String(expr_string) => self.interpret_string(expr_string),
            Expr::Inline => self.interpret_inline(),
            Expr::Drop => self.interpret_drop(),
            Expr::Query(query) => self.interpret_query(query, src_loc),
            Expr::True => Ok(ExprValue::Yaml(Yaml::Boolean(true))),
            Expr::False => Ok(ExprValue::Yaml(Yaml::Boolean(false))),
            Expr::Eq(op) => self.interpret_eq(op, src_loc),
            Expr::Ne(op) => self.interpret_ne(op, src_loc),
            Expr::Integer(integer) => self.interpret_integer(integer),
            Expr::Real(real) => self.interpret_real(real),
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

    fn query(&mut self, query: &ExprQuery, src_loc: &SourceLocationSpan) -> Result<Yaml, Error> {
        match query {
            ExprQuery::Root => Ok(self.config.clone()),
            ExprQuery::Var(name) => self.query_var(name, src_loc),
            ExprQuery::Index(objectindex) => self.query_index(objectindex, src_loc),
        }
    }

    fn query_var(&mut self, name: &str, src_loc: &SourceLocationSpan) -> Result<Yaml, Error> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.variables.get(name) {
                return Ok(value.clone());
            }
        }

        Err(errwithloc!(src_loc, "cannot find variable '{}'", name))
    }

    fn query_index(&mut self, objectindex: &ExprIndex, src_loc: &SourceLocationSpan) -> Result<Yaml, Error> {
        let index = self.interpret_expr(&objectindex.index, src_loc)?;
        let object = self.query(&objectindex.object, src_loc)?;
        match object {
            Yaml::Hash(object) => {
                let index = match index {
                    ExprValue::Yaml(yaml @ Yaml::String(_)) => yaml,
                    _ => {
                        return Err(errwithloc!(
                            src_loc,
                            "value of type {} cannot be used to index into a map",
                            Self::exp_value_type_name(&index),
                        ))
                    }
                };

                let subvalue = object.get(&index);
                match subvalue {
                    Some(subvalue) => Ok(subvalue.clone()),
                    None => Err(errwithloc!(
                        src_loc,
                        "index {} not found",
                        Self::yaml_debug_string(&index),
                    )),
                }
            }
            Yaml::Array(list) => {
                let index = match index {
                    ExprValue::Yaml(Yaml::Integer(index)) => index,
                    _ => {
                        return Err(errwithloc!(
                            src_loc,
                            "value of type {} cannot be used to index into a list",
                            Self::exp_value_type_name(&index),
                        ))
                    }
                };

                let index = usize::try_from(index)?;
                let subvalue = list.get(index);
                match subvalue {
                    Some(subvalue) => Ok(subvalue.clone()),
                    None => Err(errwithloc!(src_loc, "index {} is out of bounds", index)),
                }
            }
            _ => Err(errwithloc!(
                src_loc,
                "cannot get index {}: value type {} is not indexable",
                Self::expr_value_debug_string(&index),
                Self::yaml_type_name(&object),
            )),
        }
    }

    fn interpret_eq(&mut self, op: &ExprOpBinary, src_loc: &SourceLocationSpan) -> Result<ExprValue, Error> {
        let left = self.interpret_expr(&op.left, src_loc)?;
        let right = self.interpret_expr(&op.right, src_loc)?;
        let res = left == right;
        let res = ExprValue::Yaml(Yaml::Boolean(res));
        Ok(res)
    }

    fn interpret_ne(&mut self, op: &ExprOpBinary, src_loc: &SourceLocationSpan) -> Result<ExprValue, Error> {
        let left = self.interpret_expr(&op.left, src_loc)?;
        let right = self.interpret_expr(&op.right, src_loc)?;
        let res = left != right;
        let res = ExprValue::Yaml(Yaml::Boolean(res));
        Ok(res)
    }

    fn interpret_integer(&mut self, integer: &ExprInteger) -> Result<ExprValue, Error> {
        Ok(ExprValue::Yaml(Yaml::Integer(integer.value)))
    }

    fn interpret_real(&mut self, real: &ExprReal) -> Result<ExprValue, Error> {
        Ok(ExprValue::Yaml(Yaml::Real(real.value.clone())))
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            variables: HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn add_binding(&mut self, binding: &ExprBinding, value: &Yaml) {
        match binding {
            ExprBinding::Var(name) => self
                .scopes
                .last_mut()
                .unwrap()
                .variables
                .insert(name.as_ref().clone(), value.clone()),
        };
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
            ValueData::For(..) => Err(errwithloc!(
                value.src_loc,
                "expression value 'for' can only be used as a map key"
            )),
        }
    }

    fn expect_implicit_bool(value: ExprValue, src_loc: &SourceLocationSpan) -> Result<bool, Error> {
        // Convrert null and false to false. All other valid values are true.
        // This matches jq's semantics.
        match value {
            ExprValue::Inline => Err(errwithloc!(
                src_loc,
                "expression value 'inline' cannot be converted to a bool value"
            )),
            ExprValue::Drop => Err(errwithloc!(
                src_loc,
                "expression value 'drop' cannot be converted to a bool value"
            )),
            ExprValue::Yaml(Yaml::Boolean(false)) | ExprValue::Yaml(Yaml::Null) => Ok(false),
            ExprValue::Yaml(_) => Ok(true),
        }
    }

    fn expect_iterable(expr_value: ExprValue, src_loc: &SourceLocationSpan) -> Result<Yaml, Error> {
        match expr_value {
            ExprValue::Inline => Err(errwithloc!(src_loc, "expression value 'inline' is not iteratable")),
            ExprValue::Drop => Err(errwithloc!(src_loc, "expression value 'drop' is not iteratable")),
            ExprValue::Yaml(yaml) => match yaml {
                Yaml::Array(_) | Yaml::Hash(_) => Ok(yaml),
                _ => Err(errwithloc!(
                    src_loc,
                    "value type {} is not iteratable",
                    Self::yaml_type_name(&yaml)
                )),
            },
        }
    }

    fn expr_value_debug_string(value: &ExprValue) -> String {
        match value {
            ExprValue::Inline => "inline".to_string(),
            ExprValue::Drop => "drop".to_string(),
            ExprValue::Yaml(yaml) => Self::yaml_debug_string(yaml),
        }
    }

    fn yaml_debug_string(yaml: &Yaml) -> String {
        match yaml {
            Yaml::Real(value) => value.as_ref().clone(),
            Yaml::Integer(value) => format!("{}", value).into(),
            Yaml::String(value) => format!("{:?}", value).into(),
            Yaml::Boolean(value) => format!("{}", value).into(),
            Yaml::Array(_) => "<list>".to_string(),
            Yaml::Hash(_) => "<map>".to_string(),
            Yaml::Null => "<null>".to_string(),
        }
    }

    fn exp_value_type_name(value: &ExprValue) -> &'static str {
        match value {
            ExprValue::Inline => "inline",
            ExprValue::Drop => "drop",
            ExprValue::Yaml(yaml) => Self::yaml_type_name(yaml),
        }
    }

    fn yaml_type_name(yaml: &Yaml) -> &'static str {
        match yaml {
            Yaml::Real(_) => "number",
            Yaml::Integer(_) => "integer",
            Yaml::String(_) => "string",
            Yaml::Boolean(_) => "bool",
            Yaml::Array(_) => "list",
            Yaml::Hash(_) => "map",
            Yaml::Null => "null",
        }
    }
}
