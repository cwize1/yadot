#[derive(Clone)]
pub struct FileTemplate {
    pub docs: Vec<DocumentTemplate>,
}

#[derive(Clone)]
pub struct DocumentTemplate {
    pub node: NodeTemplate,
}

#[derive(Clone)]
pub enum NodeTemplate {
    Sequence(SequenceTemplate),
    Map(MapTemplate),
    Scaler(ScalerTemplate),
}

#[derive(Clone)]
pub struct SequenceTemplate {
    pub nodes: Vec<NodeTemplate>,
}

#[derive(Clone)]
pub struct MapTemplate {
    pub entries: Vec<MapEntryTemplate>,
}

#[derive(Clone)]
pub struct MapEntryTemplate {
    pub key: NodeTemplate,
    pub value: NodeTemplate,
}

#[derive(Clone)]
pub struct ScalerTemplate {
    pub exprs: Vec<Expr>,
}

#[derive(Clone)]
pub enum Expr {
    String(ExprString),
}

#[derive(Clone)]
pub struct ExprString {
    pub value: String,
}
