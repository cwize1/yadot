// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct FileTemplate {
    pub src_loc: SourceLocationSpan,
    pub docs: Vec<DocumentTemplate>,
}

#[derive(Clone, Debug)]
pub struct DocumentTemplate {
    pub src_loc: SourceLocationSpan,
    pub node: NodeTemplate,
}

#[derive(Clone, Debug)]
pub enum NodeTemplate {
    Sequence(SequenceTemplate),
    Map(MapTemplate),
    Scaler(ScalerTemplate),
}

#[derive(Clone, Debug)]
pub struct SequenceTemplate {
    pub src_loc: SourceLocationSpan,
    pub values: Vec<NodeTemplate>,
}

#[derive(Clone, Debug)]
pub struct MapTemplate {
    pub src_loc: SourceLocationSpan,
    pub entries: Vec<MapEntryTemplate>,
}

#[derive(Clone, Debug)]
pub struct MapEntryTemplate {
    pub key: NodeTemplate,
    pub value: NodeTemplate,
}

#[derive(Clone, Debug)]
pub struct ScalerTemplate {
    pub src_loc: SourceLocationSpan,
    pub values: Vec<ScalarTemplateValue>,
}

#[derive(Clone, Debug)]
pub enum ScalarTemplateValue {
    String(Rc<String>),
    Expr(Statement),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Expr(Expr),
    If(StatementIf),
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatementIf {
    pub condition: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    String(ExprString),
    Inline,
    Drop,
    Query(ExprQuery),
    True,
    False,
    Eq(ExprOpBinary),
    Ne(ExprOpBinary),
    Integer(ExprInteger),
    Real(ExprReal),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprString {
    pub value: Rc<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprInteger {
    pub value: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprReal {
    pub value: Rc<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprQuery {
    Root,
    Var(Rc<String>),
    Index(ExprIndex),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprOpBinary {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprIndex {
    pub object: Box<ExprQuery>,
    pub index: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceLocationSpan {
    pub filename: Rc<String>,
    pub start: SourceLocation,
    pub end: SourceLocation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceLocation {
    pub index: usize,
    pub line: usize,
    pub col: usize,
}

impl NodeTemplate {
    pub fn src_loc(&self) -> &SourceLocationSpan {
        match self {
            NodeTemplate::Sequence(SequenceTemplate { src_loc, .. })
            | NodeTemplate::Map(MapTemplate { src_loc, .. })
            | NodeTemplate::Scaler(ScalerTemplate { src_loc, .. }) => &src_loc,
        }
    }
}
