use json;
use json::JsonValue;
use std::fs;
use std::path::Path;
use pandoc;

use common;
use input_source::*;


pub struct Europeana;


struct Articles {
    paths: Vec<String>,
    // index into paths
    file_index: usize,
}

impl Articles {
    fn new(top_level: &Path) -> Self {
        let mut files_seen = Vec::new();
        Self::recurse_files(&mut files_seen, top_level);
        Articles { paths: files_seen, file_index: 0 }
    }

    pub fn recurse_files(files_read: &mut Vec<String>, directory: &Path) {
        let paths = fs ::read_dir(directory).unwrap();
        for path in paths {
            if path.is_err() {
                warn!("couldn't list contents of {:?}", path);
                continue;
            }
            let path = path.unwrap().path();
            if path.is_dir() {
                Self::recurse_files(files_read, &path)
            } else { // is a file
                if let Some(fname) = path.to_str().clone() {
                    if fname.ends_with(".json") {
                        let mut absolute_path = ::std::env::current_dir().unwrap();
                        absolute_path.push(fname);
                        files_read.push(String::from(absolute_path.to_str().unwrap()));
                    }
                } else { // error while converting path to str
                    warn!("could not decode file name of {:?}", path);
                }
            }
        }
    }
}

/// return an TransformationError; simple short-hand
#[inline]
fn mkerr(input: &str, o: Option<String>) -> Result<String> {
    Err(TransformationError::ErrorneousStructure(input.to_string(), o))
}

impl Iterator for Articles {
    type Item = Result<String>;

    // policy:
    // propagate errors directly, but skip to next file if no content could be found
    fn next(&mut self) -> Option<Result<String>> {
        if self.file_index >= self.paths.len() {
            return None
        }

        let path = get!(self.paths.get(self.file_index));
        let edition_js = match common::read_file_from_str(path) {
            Ok(x) => x,
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
                            at the top level".into(), Some(path.clone()))),
        };
        self.file_index += 1;
        Some(Ok(output))
    }
}

impl GetIterator for Europeana {
    fn iter(&self, input: &Path) -> Box<Iterator<Item=Result<String>>> {
        Box::new(Articles::new(input))
    }
}

impl Unformatter for Europeana {
    fn is_preprocessing_required(&self) -> bool {
        false
    }

    fn get_input_format(&self) -> pandoc::InputFormat {
        pandoc::InputFormat::Markdown
    }

    fn preprocess(&self, input: &str) -> Result<String> {
        Ok(input.to_string())
    }
}

