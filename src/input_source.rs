use std::path::Path;

/// Bundle all the input-specific functionality in one type
///
/// For the outside world, it's only important that there's so mething to iterate over (e.g. a
/// paragraph, an article or a book) and a preprocessing function (if required) to make processing
/// easier / to enable preprocessing in the first place.
pub trait InputSource {
    /// Return a chunk of text which should be processed at once.
    ///
    /// This is the smallest unit dispatched into the work queue and mgiht be an article, a book or
    /// something similar.
    fn get_input(input: &Path) -> Box<Iterator<Item=String>>;

    /// Reports whether preprocessing is required for this format. See prpreprocess documentation.
    fn is_preprocessing_required() -> bool;

    /// Preprocess input String to be easier to process in later steps.
    ///
    /// Pandoc, used for serializing a document, is not able to read all formatting instructions of
    /// all formats. Therefore preprocess functions may alter the content to make processing
    /// easier.
    fn preprocess(input: &str) -> Result<String, String>;
}
