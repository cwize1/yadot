use std::str::Chars;

use anyhow::Error;
use yaml_rust::{parser::Parser, Event};

use crate::yaml_template::ast::MapEntryTemplate;

use super::{ast::{DocumentTemplate, Expr, ExprString, FileTemplate, MapTemplate, NodeTemplate, ScalerTemplate, SequenceTemplate}, template_expr_parser::parse_template_expression};

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
            Event::SequenceStart(_) | Event::MappingStart(_) | Event::Scalar(_, _, _, _) | Event::Alias(_) => {
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
    // Parse MappingStart.
    let (map_start, _) = yaml_parser.next()?;
    assert!(matches!(map_start, Event::MappingStart(..)));

    // Parse entries.
    let mut entries = Vec::new();
    loop {
        let (event, _) = yaml_parser.peek()?;
        let key = match event {
            Event::SequenceStart(_) | Event::MappingStart(_) | Event::Scalar(_, _, _, _) | Event::Alias(_) => {
                parse_node(yaml_parser)?
            },
            Event::MappingEnd => break,
            _ => unreachable!(),
        };

        let (event, _) = yaml_parser.peek()?;
        let value = match event {
            Event::SequenceStart(_) | Event::MappingStart(_) | Event::Scalar(_, _, _, _) | Event::Alias(_) => {
                parse_node(yaml_parser)?
            },
            _ => unreachable!(),
        };

        let entry = MapEntryTemplate{
            key,
            value,
        };
        entries.push(entry);
    }

    // Parse MappingEnd.
    let (seq_end, _) = yaml_parser.next()?;
    assert_eq!(seq_end, Event::MappingEnd);

    // Return result.
    let map = MapTemplate{
        entries,
    };
    Ok(map)
}

fn parse_scaler(yaml_parser: &mut Parser<Chars>) -> Result<ScalerTemplate, Error> {
    // Parse Scalar.
    let (scalar, _) = yaml_parser.next()?;
    let Event::Scalar(value, _, _, _) = scalar else { unreachable!() };

    let mut curr_index = 0;
    let mut exprs = Vec::new();
    loop {
        let template_expr_index = value[curr_index..].find("${{");
        let Some(template_expr_index) = template_expr_index else {
            break
        };

        // Add non-template string characters.
        if template_expr_index > curr_index {
            let non_template_str = value[curr_index..template_expr_index].to_string();
            let non_template_expr = Expr::String(ExprString{
                value: non_template_str,
            });
            exprs.push(non_template_expr);
        }

        // Add template string expression.
        let expr_str = &value[template_expr_index..];
        let (expr, end) = parse_template_expression(expr_str)?;
        exprs.push(expr);

        curr_index = template_expr_index + end;
    }

    // Add non-template string characters.
    if value.len() > curr_index {
        let non_template_str = value[curr_index..].to_string();
        let non_template_expr = Expr::String(ExprString{
            value: non_template_str,
        });
        exprs.push(non_template_expr);
    }

    let scalar = ScalerTemplate{
        exprs,
    };
    Ok(scalar)
}
