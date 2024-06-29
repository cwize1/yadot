use std::str::Chars;

use anyhow::Error;
use yaml_rust::{parser::Parser, Event};

use super::ast::{DocumentTemplate, FileTemplate, MapTemplate, NodeTemplate, ScalerTemplate, SequenceTemplate};

pub fn parse_yaml_template(input: &str) -> Result<FileTemplate, Error> {
    let yaml_parser = &mut Parser::new(input.chars());

    // Parse StreamStart.
    let (evt_strm_start, _) = yaml_parser.next()?;
    assert_eq!(evt_strm_start, Event::StreamStart);

    // Parse docs.
    let mut docs = Vec::new();
    loop {
        let (event, _) = yaml_parser.peek()?;
        match event {
            Event::DocumentStart => {
                let doc = parse_yaml_doc(yaml_parser)?;
                docs.push(doc);
            },
            Event::StreamEnd => break,
            _ => unreachable!(),
        }
    }

    // Parse StreamEnd.
    let (evt_strm_end, _) = yaml_parser.next()?;
    assert_eq!(evt_strm_end, Event::StreamEnd);

    // Return result.
    let file = FileTemplate {
        docs: docs,
    };
    Ok(file)
}

fn parse_yaml_doc(yaml_parser: &mut Parser<Chars>) -> Result<DocumentTemplate, Error> {
    // Parse DocumentStart.
    let (doc_start, _) = yaml_parser.next()?;
    assert_eq!(doc_start, Event::DocumentStart);

    // Parse node.
    let node = parse_node(yaml_parser)?;

    // Parse DocumentEnd.
    let (doc_start, _) = yaml_parser.next()?;
    assert_eq!(doc_start, Event::DocumentEnd);

    // Return result.
    let doc = DocumentTemplate{
        node,
    };
    Ok(doc)
}

fn parse_node(yaml_parser: &mut Parser<Chars>) -> Result<NodeTemplate, Error> {
    let (event, _) = yaml_parser.peek()?;
    match event {
        Event::SequenceStart(_) => {
            let sequence = parse_sequence(yaml_parser)?;
            Ok(NodeTemplate::Sequence(sequence))
        },
        Event::MappingStart(_) => {
            let map = parse_mapping(yaml_parser)?;
            Ok(NodeTemplate::Map(map))
        },
        Event::Scalar(_, _, _, _) => {
            let scaler = parse_scaler(yaml_parser)?;
            Ok(NodeTemplate::Scaler(scaler))
        },
        Event::Alias(_) => todo!(),
        _ => unreachable!(),
    }
}

fn parse_sequence(yaml_parser: &mut Parser<Chars>) -> Result<SequenceTemplate, Error> {
    // Parse SequenceStart.
    let (seq_start, _) = yaml_parser.next()?;
    assert!(matches!(seq_start, Event::SequenceStart(..)));

    // Parse nodes.
    let mut nodes = Vec::new();
    loop {
        let (event, _) = yaml_parser.peek()?;
        match event {
            Event::DocumentStart => {
                let node = parse_node(yaml_parser)?;
                nodes.push(node);
            },
            Event::SequenceEnd => break,
            _ => unreachable!(),
        }
    }

    // Parse SequenceEnd.
    let (seq_end, _) = yaml_parser.next()?;
    assert_eq!(seq_end, Event::SequenceEnd);

    // Return result.
    let seq = SequenceTemplate{
        nodes,
    };
    Ok(seq)
}

fn parse_mapping(yaml_parser: &mut Parser<Chars>) -> Result<MapTemplate, Error> {
    todo!()
}

fn parse_scaler(yaml_parser: &mut Parser<Chars>) -> Result<ScalerTemplate, Error> {
    todo!()
}
