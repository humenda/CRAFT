/// This module parses the code civil in MarkDown format, as published by Steeve
/// Morin. It has to be in a separate directory containing files ending on .md.
use std::fs;
use std::path::Path;
use pandoc;

use common;
use input_source::{GetIterator, Result, Unformatter};



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
        let paths = fs::read_dir(directory).unwrap();
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
                    if fname.ends_with(".md") {
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

impl Iterator for Articles {
    type Item = Result<String>;

    // policy:
    // propagate errors directly, but skip to next file if no content could be found
    fn next(&mut self) -> Option<Result<String>> {
        if self.file_index >= self.paths.len() {
            return None
        }

        let path = get!(self.paths.get(self.file_index));
        self.file_index += 1;
        Some(common::read_file_from_str(path))
    }
}



/// Code Civil input parser
///
/// The code civil comes in pure markdown format and is hence easy to handle. In
/// fact, all the magic is handled by markdown. This struct only makes sure that
/// files are loaded correctly.
pub struct CodeCivil;

impl GetIterator for CodeCivil {
    fn iter(&self, input: &Path) -> Box<Iterator<Item=Result<String>>> {
        Box::new(Articles::new(input))
    }
}

impl Unformatter for CodeCivil {
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

