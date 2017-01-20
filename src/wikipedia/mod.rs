mod articles;
mod preprocessor;

use std::path::Path;

pub use self::articles::*;
pub use self::preprocessor::*;

use super::input_source;
use super::input_source::Result;

use pandoc;

/// For documentation, please see the type Articles and MediawikiPreprocessor
pub struct Wikipedia;

impl input_source::GetIterator for Wikipedia {
    fn iter(&self, dst: &Path) -> Box<Iterator<Item=Result<String>>> {
        Box::new(parser_from_file(dst))
    }
}

impl<'a> input_source::Unformatter for Wikipedia {
    fn is_preprocessing_required(&self) -> bool {
        true
    }

    fn get_input_format(&self) -> pandoc::InputFormat {
        return pandoc::InputFormat::MediaWiki;
    }

    fn preprocess(&self, input: &str) -> Result<String> {
        let mut preproc = MediawikiPreprocessor::new(input);
        preproc.preprocess()
    }
}
