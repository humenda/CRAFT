/// This module parses the code civil in MarkDown format, as published by Steeve
/// Morin. It has to be in a separate directory containing files ending on .md.
use std::path::{Path};
use pandoc;

use common;
use input_source::{GetIterator, Result, Unformatter};


/// Code Civil input parser
///
/// The code civil comes in pure markdown format and is hence easy to handle. In
/// fact, all the magic is handled by markdown. This struct only makes sure that
/// files are loaded correctly.
pub struct CodeCivil;

impl GetIterator for CodeCivil {
    fn iter(&self, input: &Path, _: Option<String>) -> Box<Iterator<Item=Result<String>>> {
        common::read_files(input.into(), ".md".into())
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

