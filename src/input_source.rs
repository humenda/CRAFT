use json;
use std::error::Error;
use std::io;
use pandoc;

pub type Result<T> = ::std::result::Result<T, TransformationError>;


#[derive(Debug)]
pub enum TransformationError {
    /// Save an IO error and the path to the file where the io::Error occurred (if possible)
    IoError(io::Error, Option<String>),
    /// Structural errors in the file format; may contain a message and an otpional path
    ErrorneousStructure(String, Option<String>),
    JsonError(json::Error),
    /// XML parser errors
    XmlParserERrror(::xml::reader::Error),
    /// invalid input arguments, e.g. a invalid language
    InvalidInputArguments(String),
    /// encoding issues
    EncodingError(String),
}

impl ::std::fmt::Display for TransformationError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            TransformationError::ErrorneousStructure(ref msg, ref path) => write!(f,
                          "{}{}", msg, path.clone().unwrap_or_else(String::new)),
            TransformationError::InvalidInputArguments(ref msg) => write!(f, "{}", msg),
            TransformationError::IoError(ref e, ref path) => {
                write!(f, "{}: ", path.clone().unwrap_or(String::from("<no path>")))?;
                Ok(e.fmt(f)?)
            },
            TransformationError::JsonError(ref e) => e.fmt(f),
            TransformationError::EncodingError(ref e) => write!(f, "{}", e),
            TransformationError::XmlParserERrror(ref e) => e.fmt(f),
        }
    }
}

impl Error for TransformationError {
    fn description(&self) -> &str {
        match *self {
            TransformationError::ErrorneousStructure(_, _) => "invalid structure",
            TransformationError::InvalidInputArguments(_) => "received invalid input arguments",
            TransformationError::EncodingError(_) => "invalid encoding",
            TransformationError::IoError(ref err, _) => err.description(),
            TransformationError::JsonError(ref err) => err.description(),
            TransformationError::XmlParserERrror(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TransformationError::IoError(ref err, _) => err.cause(),
            TransformationError::JsonError(ref err) => err.cause(),
            TransformationError::XmlParserERrror(ref err) => err.cause(),
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

/// allow hassle-free coercion from xml::reader::Error
impl From<::xml::reader::Error> for TransformationError {
    fn from(err: ::xml::reader::Error) -> TransformationError {
        TransformationError::XmlParserERrror(err)
    }
}

impl From<json::Error> for TransformationError {
    fn from(err: json::Error) -> TransformationError {
        TransformationError::JsonError(err)
    }
}

impl From<::zip::result::ZipError> for TransformationError {
    fn from(err: ::zip::result::ZipError) -> TransformationError {
        use ::zip::result::ZipError;
        match err {
            ZipError::FileNotFound => TransformationError::IoError(io::Error::new(
                    io::ErrorKind::NotFound, "file not found"), None),
            ZipError::Io(e) => TransformationError::IoError(e, None),
            ZipError::InvalidArchive(ref msg) => TransformationError::ErrorneousStructure(
                    format!("Invalid zip file: {}", msg), None),
            ZipError::UnsupportedArchive(ref msg) => TransformationError::ErrorneousStructure(
                    format!("Unsupported zip archive format: {}", msg), None),

        }
    }
}



/// Return the corresponding iterator for a given input source.
/// Strip formatting from a document
///
/// This trait provides methods to remove formatting from input sources. It is rather specific to
/// Pandoc, because this is used to transform concrete formats into an abstract document AST.
pub trait Unformatter {
    /// Reports whether preprocessing is required for this format.
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

