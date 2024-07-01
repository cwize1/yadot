mod template_expr_lexer;
mod template_expr_parser;

use std::str::Chars;

use anyhow::Error;
use yaml_rust::{parser::Parser as YamlParser, Event};

use crate::ast::{
    DocumentTemplate, FileTemplate, MapEntryTemplate, MapTemplate, NodeTemplate, ScalarTemplateValue, ScalerTemplate,
    SequenceTemplate,
};

use template_expr_parser::TemplateExprParser;

pub struct Parser {
    expr_parser: TemplateExprParser,
}

impl Parser {
    pub fn new() -> Parser {
        let expr_parser = TemplateExprParser::new();
        Parser { expr_parser }
    }

    pub fn parse(&self, input: &str) -> Result<FileTemplate, Error> {
        let yaml_parser = &mut YamlParser::new(input.chars());

        // Parse StreamStart.
        let (evt_strm_start, _) = yaml_parser.next()?;
        assert_eq!(evt_strm_start, Event::StreamStart);

        // Parse docs.
        let mut docs = Vec::new();
        loop {
            let (event, _) = yaml_parser.peek()?;
            match event {
                Event::DocumentStart => {
                    let doc = self.parse_yaml_doc(yaml_parser)?;
                    docs.push(doc);
                }
                Event::StreamEnd => break,
                _ => unreachable!(),
            }
        }

        // Parse StreamEnd.
        let (evt_strm_end, _) = yaml_parser.next()?;
        assert_eq!(evt_strm_end, Event::StreamEnd);

        // Return result.
        let file = FileTemplate { docs: docs };
        Ok(file)
    }

    fn parse_yaml_doc(&self, yaml_parser: &mut YamlParser<Chars>) -> Result<DocumentTemplate, Error> {
        // Parse DocumentStart.
        let (doc_start, _) = yaml_parser.next()?;
        assert_eq!(doc_start, Event::DocumentStart);

        // Parse node.
        let node = self.parse_node(yaml_parser)?;

        // Parse DocumentEnd.
        let (doc_start, _) = yaml_parser.next()?;
        assert_eq!(doc_start, Event::DocumentEnd);

        // Return result.
        let doc = DocumentTemplate { node };
        Ok(doc)
    }

    fn parse_node(&self, yaml_parser: &mut YamlParser<Chars>) -> Result<NodeTemplate, Error> {
        let (event, _) = yaml_parser.peek()?;
        match event {
            Event::SequenceStart(_) => {
                let sequence = self.parse_sequence(yaml_parser)?;
                Ok(NodeTemplate::Sequence(sequence))
            }
            Event::MappingStart(_) => {
                let map = self.parse_mapping(yaml_parser)?;
                Ok(NodeTemplate::Map(map))
            }
            Event::Scalar(_, _, _, _) => {
                let scaler = self.parse_scaler(yaml_parser)?;
                Ok(NodeTemplate::Scaler(scaler))
            }
            Event::Alias(_) => todo!(),
            _ => unreachable!(),
        }
    }

    fn parse_sequence(&self, yaml_parser: &mut YamlParser<Chars>) -> Result<SequenceTemplate, Error> {
        // Parse SequenceStart.
        let (seq_start, _) = yaml_parser.next()?;
        assert!(matches!(seq_start, Event::SequenceStart(..)));

        // Parse nodes.
        let mut values = Vec::new();
        loop {
            let (event, _) = yaml_parser.peek()?;
            match event {
                Event::SequenceStart(_) | Event::MappingStart(_) | Event::Scalar(_, _, _, _) | Event::Alias(_) => {
                    let value = self.parse_node(yaml_parser)?;
                    values.push(value);
                }
                Event::SequenceEnd => break,
                _ => unreachable!(),
            }
        }

        // Parse SequenceEnd.
        let (seq_end, _) = yaml_parser.next()?;
        assert_eq!(seq_end, Event::SequenceEnd);

        // Return result.
        let seq = SequenceTemplate { values };
        Ok(seq)
    }

    fn parse_mapping(&self, yaml_parser: &mut YamlParser<Chars>) -> Result<MapTemplate, Error> {
        // Parse MappingStart.
        let (map_start, _) = yaml_parser.next()?;
        assert!(matches!(map_start, Event::MappingStart(..)));

        // Parse entries.
        let mut entries = Vec::new();
        loop {
            let (event, _) = yaml_parser.peek()?;
            let key = match event {
                Event::SequenceStart(_) | Event::MappingStart(_) | Event::Scalar(_, _, _, _) | Event::Alias(_) => {
                    self.parse_node(yaml_parser)?
                }
                Event::MappingEnd => break,
                _ => unreachable!(),
            };

            let (event, _) = yaml_parser.peek()?;
            let value = match event {
                Event::SequenceStart(_) | Event::MappingStart(_) | Event::Scalar(_, _, _, _) | Event::Alias(_) => {
                    self.parse_node(yaml_parser)?
                }
                _ => unreachable!(),
            };

            let entry = MapEntryTemplate { key, value };
            entries.push(entry);
        }

        // Parse MappingEnd.
        let (seq_end, _) = yaml_parser.next()?;
        assert_eq!(seq_end, Event::MappingEnd);

        // Return result.
        let map = MapTemplate { entries };
        Ok(map)
    }

    fn parse_scaler(&self, yaml_parser: &mut YamlParser<Chars>) -> Result<ScalerTemplate, Error> {
        // Parse Scalar.
        let (scalar, _) = yaml_parser.next()?;
        let Event::Scalar(value, _, _, _) = scalar else {
            unreachable!()
        };

        let mut curr_index = 0;
        let mut values = Vec::new();
        loop {
            // Find next template expression.
            let template_expr_index = value[curr_index..].find("${{");
            let Some(template_expr_index) = template_expr_index else {
                break;
            };

            // Add non-template string characters.
            if template_expr_index > curr_index {
                let non_template_str = value[curr_index..template_expr_index].to_string();
                let value = ScalarTemplateValue::String(non_template_str);
                values.push(value);
            }

            // Add template expression.
            let expr_str = &value[template_expr_index..];
            let (expr, end) = self.expr_parser.parse(expr_str)?;
            let value = ScalarTemplateValue::Expr(expr);
            values.push(value);

            curr_index = template_expr_index + end;
        }

        // Add non-template string characters.
        if value.len() > curr_index {
            let non_template_str = value[curr_index..].to_string();
            let value = ScalarTemplateValue::String(non_template_str);
            values.push(value);
        }

        let scalar = ScalerTemplate { values };
        Ok(scalar)
    }
}
