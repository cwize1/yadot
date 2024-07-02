use std::{
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
};

use anyhow::Error;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

pub fn yaml_emit_to_string(docs: &Vec<Yaml>) -> Result<String, Error> {
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    for doc in docs {
        emitter.dump(doc)?;
    }
    Ok(out_str)
}

pub fn yaml_emit_to_file(docs: &Vec<Yaml>, filename: &PathBuf) -> Result<(), Error> {
    let out = yaml_emit_to_string(docs)?;
    fs::write(filename, out)?;
    Ok(())
}

pub fn yaml_load_from_file(filename: &PathBuf) -> Result<Vec<Yaml>, Error> {
    let tests_data_str = fs::read_to_string(&filename).unwrap();
    let tests_data_docs = YamlLoader::load_from_str(&tests_data_str).unwrap();
    Ok(tests_data_docs)
}
