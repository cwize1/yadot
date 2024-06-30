use std::borrow::Cow;

use anyhow::Error;
use yaml_rust::{yaml::Hash, Yaml};

use crate::ast::{
    DocumentTemplate, Expr, ExprString, FileTemplate, MapTemplate, NodeTemplate, ScalarTemplateValue, ScalerTemplate,
    SequenceTemplate,
};

pub struct InterpreterRun {}

impl InterpreterRun {
    pub fn new() -> InterpreterRun {
        InterpreterRun {}
    }

    pub fn interpret_file(&mut self, file_templ: &FileTemplate) -> Result<Vec<Yaml>, Error> {
        let mut docs = Vec::new();
        for doc_templ in &file_templ.docs {
            let doc = self.interpret_doc(doc_templ)?;
            docs.push(doc);
        }

        Ok(docs)
    }

    fn interpret_doc(&mut self, doc_templ: &DocumentTemplate) -> Result<Yaml, Error> {
        let node = self.interpret_node(&doc_templ.node)?;
        Ok(node)
    }

    fn interpret_node(&mut self, node_templ: &NodeTemplate) -> Result<Yaml, Error> {
        match node_templ {
            NodeTemplate::Sequence(seq_templ) => self.interpret_seq(seq_templ),
            NodeTemplate::Map(map_templ) => self.interpret_map(map_templ),
            NodeTemplate::Scaler(scalar_templ) => self.interpret_scalar(scalar_templ),
        }
    }

    fn interpret_seq(&mut self, seq_templ: &SequenceTemplate) -> Result<Yaml, Error> {
        let mut values = Vec::new();
        for value_templ in &seq_templ.values {
            let value = self.interpret_node(value_templ)?;
            values.push(value);
        }

        let seq = Yaml::Array(values);
        Ok(seq)
    }

    fn interpret_map(&mut self, map_templ: &MapTemplate) -> Result<Yaml, Error> {
        let mut entries = Hash::new();
        for entry_templ in &map_templ.entries {
            let key = self.interpret_node(&entry_templ.key)?;
            let value = self.interpret_node(&entry_templ.value)?;
            entries.insert(key, value);
        }

        let map = Yaml::Hash(entries);
        Ok(map)
    }

    fn interpret_scalar(&mut self, scalar_templ: &ScalerTemplate) -> Result<Yaml, Error> {
        let mut string = String::new();

        for value_templ in &scalar_templ.values {
            match value_templ {
                ScalarTemplateValue::String(substring) => string.push_str(substring),
                ScalarTemplateValue::Expr(expr) => {
                    let expr_string = self.interpret_expr(expr)?;
                    string.push_str(&expr_string);
                }
            }
        }

        let scalar = Yaml::String(string);
        Ok(scalar)
    }

    fn interpret_expr<'a>(&mut self, expr: &'a Expr) -> Result<Cow<'a, str>, Error> {
        match expr {
            Expr::String(expr_string) => self.interpret_expr_string(expr_string),
            Expr::Inline => todo!(),
            Expr::Drop => todo!(),
        }
    }

    fn interpret_expr_string<'a>(&mut self, expr_string: &'a ExprString) -> Result<Cow<'a, str>, Error> {
        Ok(Cow::Borrowed(&expr_string.value))
    }
}
