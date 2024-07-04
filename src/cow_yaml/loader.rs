use std::{rc::Rc, str::Chars};

use anyhow::{anyhow, Error};
use linked_hash_map::LinkedHashMap;
use yaml_rust::{
    parser::Parser as YamlParser,
    scanner::{TScalarStyle, TokenType},
    Event,
};

use super::Yaml;

pub fn parse_yaml_str(input: &str) -> Result<Vec<Yaml>, Error> {
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
                let doc = parse_doc(yaml_parser)?;
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
    let docs = docs;
    Ok(docs)
}

fn parse_doc(yaml_parser: &mut YamlParser<Chars>) -> Result<Yaml, Error> {
    // Parse DocumentStart.
    let (doc_start, _) = yaml_parser.next()?;
    assert_eq!(doc_start, Event::DocumentStart);

    // Parse node.
    let node = parse_node(yaml_parser)?;

    // Parse DocumentEnd.
    let (doc_start, _) = yaml_parser.next()?;
    assert_eq!(doc_start, Event::DocumentEnd);

    // Return result.
    Ok(node)
}

fn parse_node(yaml_parser: &mut YamlParser<Chars>) -> Result<Yaml, Error> {
    let (event, _) = yaml_parser.peek()?;
    match event {
        Event::SequenceStart(_) => parse_sequence(yaml_parser),
        Event::MappingStart(_) => parse_mapping(yaml_parser),
        Event::Scalar(_, _, _, _) => parse_scaler(yaml_parser),
        Event::Alias(_) => return Err(anyhow!("yaml aliases not supported")),
        _ => unreachable!(),
    }
}

fn parse_sequence(yaml_parser: &mut YamlParser<Chars>) -> Result<Yaml, Error> {
    // Parse SequenceStart.
    let (seq_start, _) = yaml_parser.next()?;
    assert!(matches!(seq_start, Event::SequenceStart(..)));

    // Parse nodes.
    let mut values = Vec::new();
    loop {
        let (event, _) = yaml_parser.peek()?;
        match event {
            Event::SequenceStart(_) | Event::MappingStart(_) | Event::Scalar(_, _, _, _) | Event::Alias(_) => {
                let value = parse_node(yaml_parser)?;
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
    let seq = Yaml::Array(Rc::new(values));
    Ok(seq)
}

fn parse_mapping(yaml_parser: &mut YamlParser<Chars>) -> Result<Yaml, Error> {
    // Parse MappingStart.
    let (map_start, _) = yaml_parser.next()?;
    assert!(matches!(map_start, Event::MappingStart(..)));

    // Parse entries.
    let mut map = LinkedHashMap::new();
    loop {
        let (event, _) = yaml_parser.peek()?;
        let key = match event {
            Event::SequenceStart(_) | Event::MappingStart(_) | Event::Scalar(_, _, _, _) | Event::Alias(_) => {
                parse_node(yaml_parser)?
            }
            Event::MappingEnd => break,
            _ => unreachable!(),
        };

        let (event, _) = yaml_parser.peek()?;
        let value = match event {
            Event::SequenceStart(_) | Event::MappingStart(_) | Event::Scalar(_, _, _, _) | Event::Alias(_) => {
                parse_node(yaml_parser)?
            }
            _ => unreachable!(),
        };

        map.insert(key, value);
    }

    // Parse MappingEnd.
    let (map_end, _) = yaml_parser.next()?;
    assert_eq!(map_end, Event::MappingEnd);

    // Return result.
    let map = Yaml::Hash(Rc::new(map));
    Ok(map)
}

fn parse_scaler(yaml_parser: &mut YamlParser<Chars>) -> Result<Yaml, Error> {
    // Parse Scalar.
    let (scalar, _) = yaml_parser.next()?;
    let Event::Scalar(value, style, _, tag) = scalar else {
        unreachable!()
    };

    match (style, tag) {
        (TScalarStyle::Plain, Some(TokenType::Tag(..))) => Err(anyhow!("yaml tags are not supported")),
        (TScalarStyle::Plain, None) => {
            let yaml = yaml_rust::Yaml::from_str(&value);
            match yaml {
                yaml_rust::Yaml::Real(value) => Ok(Yaml::Real(Rc::new(value))),
                yaml_rust::Yaml::Integer(value) => Ok(Yaml::Integer(value)),
                yaml_rust::Yaml::String(value) => Ok(Yaml::String(Rc::new(value))),
                yaml_rust::Yaml::Boolean(value) => Ok(Yaml::Boolean(value)),
                yaml_rust::Yaml::Null => Ok(Yaml::Null),
                _ => unreachable!(),
            }
        }
        _ => Ok(Yaml::String(Rc::new(value))),
    }
}
