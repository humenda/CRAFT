use json;
use json::JsonValue;
use std::path::{Path};

use common;
use input_source::*;


pub struct Europeana;


/// Iterator, which parses the content out of a JSON file
pub struct Articles {
    paths: Box<Iterator<Item=Result<Entity>>>,
}

impl Articles {
    pub fn new(top_level: &Path) -> Self {
        Articles { paths: common::read_files(top_level.into(), "json".into()) }
    }
}

/// return a TransformationError; simple short-hand
#[inline]
fn mkerr(input: &str, pos: PositionType) -> Option<Result<Entity>> {
    Some(Err(TransformationError::ErrorneousStructure(input.to_string(), pos)))
}

impl Iterator for Articles {
    type Item = Result<Entity>;

    // policy:
    // propagate errors directly, but skip to next file if no content could be found
    fn next(&mut self) -> Option<Self::Item> {
        let edition_js = trysome!(get!(self.paths.next()));
        let meta = trysome!(json::parse(&edition_js.content).map_err(|e|
            TransformationError::JsonError(e, edition_js.position.clone())));
        let mut output = String::new();
        match meta {
            JsonValue::Object(ref obj) => match obj.get("contentAsText") {
                Option::Some(&JsonValue::Array(ref values)) => for text in values {
                    output.push_str(&text.to_string());
                },
                _ => return mkerr("Expected a JSON array underneath \
                       \"contextAsText\" key", edition_js.position),
            },
            _ => return mkerr("expected JSON document with an Object at \
                the top level".into(), edition_js.position),
        };
        Some(Ok(Entity { content: output, position: edition_js.position }))
    }
}


