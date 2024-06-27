#[derive(Clone)]
pub struct FileTemplate {
    pub documents: Vec<DocumentTemplate>,
}

#[derive(Clone)]
pub struct DocumentTemplate {
    pub node: NodeTemplate,
}

#[derive(Clone)]
pub enum NodeTemplate {
    Sequence(SequenceTemplate),
    Mapping(MappingTemplate),
    Scaler(ScalerTemplate),
}

#[derive(Clone)]
pub struct SequenceTemplate {
}

#[derive(Clone)]
pub struct MappingTemplate {
}

#[derive(Clone)]
pub struct ScalerTemplate {
}
