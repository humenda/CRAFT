use json;
use json::JsonValue;
use std::path::{Path};

use common;
use input_source::*;


pub struct Europeana;


/// Iterator, which parses the content out of a JSON file
struct Articles {
    paths: Box<Iterator<Item=Result<String>>>,
}

impl Articles {
    fn new(top_level: &Path) -> Self {
        Articles { paths: common::read_files(top_level.into(), "json".into()) }
    }
}

/// return a TransformationError; simple short-hand
#[inline]
fn mkerr(input: &str, path: Option<String>) -> Result<String> {
    Err(TransformationError::ErrorneousStructure(input.to_string(), path))
}

impl Iterator for Articles {
    type Item = Result<String>;

    // policy:
    // propagate errors directly, but skip to next file if no content could be found
    fn next(&mut self) -> Option<Result<String>> {
        let edition_js = match get!(self.paths.next()) {
            Ok(p) => p,
            Err(e) => return Some(Err(e)),
        };
        let meta = match json::parse(&edition_js) {
            Ok(x) => x,
            Err(e) => return Some(Err(TransformationError::JsonError(e))),
        };
        let mut output = String::new();
        match meta {
            JsonValue::Object(ref obj) => match obj.get("contentAsText") {
                Option::Some(&JsonValue::Array(ref values)) => for text in values {
                    output.push_str(&text.to_string());
                },
                _ => return Some(mkerr("Expected a JSON array underneath \
                       \"contextAsText\" key", None)),
            },
            _ => return Some(mkerr("expected JSON document with an Object \
                            at the top level".into(), None)),
        };
        Some(Ok(output))
    }
}

impl GetIterator for Europeana {
    fn iter(&self, input: &Path, _: Option<String>) -> Box<Iterator<Item=Result<String>>> {
        Box::new(Articles::new(input))
    }
}

