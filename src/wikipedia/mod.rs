mod articles;
mod preprocessor;

use std::path::Path;

pub use self::articles::*;
pub use self::preprocessor::*;

use super::input_source;
use super::input_source::Result;

/// For documentation, please see the type Articles and MediawikiPreprocessor
pub struct Wikipedia;

impl input_source::InputSource for Wikipedia {
    fn get_input(&self, dst: &Path) -> Box<Iterator<Item=Result<String>>> {
        Box::new(parser_from_file(dst))
    }

    fn is_preprocessing_required(&self) -> bool {
        true
    }

    fn preprocess(&self, input: &str) -> Result<String> {
        let mut preproc = MediawikiPreprocessor::new(input);
        preproc.preprocess()
    }
}
