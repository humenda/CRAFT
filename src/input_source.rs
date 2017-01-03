use json;
use std::error::Error;
use std::io;
use std::path::Path;

use pandoc;

pub type Result<T> = ::std::result::Result<T, TransformationError>;

#[derive(Debug)]
pub enum TransformationError {
    /// Save an IO error and the path to the file where the io::Error occurred (if possible)
    IoError(io::Error, Option<String>),
    /// STructural errors; may contain a message and an otpional path
    ErrorneousStructure(String, Option<String>),
    JsonError(json::Error)
}

impl ::std::fmt::Display for TransformationError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            TransformationError::ErrorneousStructure(ref msg, ref path) => write!(f,
                          "{}{}", msg, path.clone().unwrap_or_else(String::new)),
            TransformationError::IoError(ref e, ref path) => {
                write!(f, "{}: ", path.clone().unwrap_or(String::from("<no path>")))?;
                Ok(e.fmt(f)?)
            },
            TransformationError::JsonError(ref e) => e.fmt(f),
        }
    }
}

impl Error for TransformationError {
    fn description(&self) -> &str {
        match *self {
            TransformationError::ErrorneousStructure(_, _) => "invalid structure",
            TransformationError::IoError(ref err, _) => err.description(),
            TransformationError::JsonError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TransformationError::IoError(ref err, _) => err.cause(),
            TransformationError::JsonError(ref err) => err.cause(),
            _ => None,
        }
    }
}

/// allow seamless coercion from io::Error 
impl From<::std::io::Error> for TransformationError {
    fn from(err: ::std::io::Error) -> TransformationError {
        TransformationError::IoError(err, None)
    }
}

impl From<json::Error> for TransformationError {
    fn from(err: json::Error) -> TransformationError {
        TransformationError::JsonError(err)
    }
}

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
    fn get_input(&self, input: &Path) -> Box<Iterator<Item=Result<String>>>;

    /// Reports whether preprocessing is required for this format. See prpreprocess documentation.
    fn is_preprocessing_required(&self) -> bool;

    /// Get input format for Pandoc
    ///
    /// Pandoc transforms the input format to an AST, which is transformed to plain text. To be
    /// able to do the transformation, the input type has to be known.
    fn get_input_format(&self) -> pandoc::InputFormat;

    /// Preprocess input String to be easier to process in later steps.
    ///
    /// Pandoc, used for serializing a document, is not able to read all formatting instructions of
    /// all formats. Therefore preprocess functions may alter the content to make processing
    /// easier. Preprocessing functions might also strip parts of the documents, which are not
    /// intended for the corpus.
    fn preprocess(&self, input: &str) -> Result<String>;
}
