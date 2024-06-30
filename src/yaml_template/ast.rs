#[derive(Clone, Debug)]
pub struct FileTemplate {
    pub docs: Vec<DocumentTemplate>,
}

#[derive(Clone, Debug)]
pub struct DocumentTemplate {
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
    pub nodes: Vec<NodeTemplate>,
}

#[derive(Clone, Debug)]
pub struct MapTemplate {
    pub entries: Vec<MapEntryTemplate>,
}

#[derive(Clone, Debug)]
pub struct MapEntryTemplate {
    pub key: NodeTemplate,
    pub value: NodeTemplate,
}

#[derive(Clone, Debug)]
pub struct ScalerTemplate {
    pub exprs: Vec<Expr>,
}

#[derive(Clone, Debug)]
pub enum Expr {
    String(ExprString),
}

#[derive(Clone, Debug)]
pub struct ExprString {
    pub value: String,
}
