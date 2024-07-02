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
    String(String),
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprString {
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprQuery {
    Root,
    ObjectIndex(ExprObjectIndex),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprIdent {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprObjectIndex {
    pub object: Box<ExprQuery>,
    pub index: ExprIdent,
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
