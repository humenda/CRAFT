use json;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use pandoc;

pub type Result<T> = ::std::result::Result<T, TransformationError>;

/// Position within a data set.
///
/// The PositionType specifies the position (and its type) of a an entity. It is
/// a rough estimate to help users and developers find problems in the input data
/// or in the program.
#[derive(Clone, Debug)]
pub enum PositionType {
    /// save row and column
    InFile(PathBuf, u64, u64),
    /// save file name
    InDirectory(PathBuf),
    /// No information available.
    None,
}

impl PositionType {
    /// Build a string representation for the contained position information.
    ///
    /// -   Paths are converted into strings.
    /// -   Row/column information are formatted as `row:col`.
    /// -   None remains none.
    pub fn to_string(&self) -> Option<String> {
        match self {
            &PositionType::InDirectory(ref p) => p.to_str().map(|x| x.into()),
            &PositionType::InFile(ref p, ref r, ref c) => Some(format!("{}: {}:{}",
                    p.to_string_lossy(), r, c)),
            &PositionType::None => None
        }
    }

    /// Return PositionType::InDirectory, if a path exists, else PositionType::None
    pub fn from_path(path: &Option<PathBuf>) -> PositionType {
        match path {
            &Some(ref p) => PositionType::InDirectory(p.clone()),
            &None => PositionType::None
        }
    }
}

impl ::std::fmt::Display for PositionType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self.to_string() {
            Some(s) => write!(f, "{}", s),
            None => write!(f, ""),
        }
    }
}



/// Wrapper error type.
#[derive(Debug)]
pub enum TransformationError {
    /// Save an IO error and the path to the file where the io::Error occurred (if possible)
    IoError(io::Error, PositionType),
    /// Structural errors in the file format; may contain a message and an optional path
    ErrorneousStructure(String, PositionType),
    JsonError(json::Error, PositionType),
    /// XML parser errors
    XmlParserERrror(::xml::reader::Error, PositionType),
    /// encoding issues
    EncodingError(String, PositionType),
    /// invalid language code, missing two-letter identifier, etc.
    InvalidLanguageError(String, String, PositionType)
}


impl TransformationError {
    pub fn inject_position(&mut self, pos: PositionType) {
        match self {
            &mut TransformationError::IoError(_, ref mut p) => *p = pos,
            &mut TransformationError::ErrorneousStructure(_, ref mut p) => *p = pos,
            &mut TransformationError::JsonError(_, ref mut p) => *p = pos,
            &mut TransformationError::XmlParserERrror(_, ref mut p) => *p = pos,
            &mut TransformationError::EncodingError(_, ref mut p) => *p = pos,
            _ => (),
        }
    }
}

impl ::std::fmt::Display for TransformationError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            TransformationError::ErrorneousStructure(ref msg, ref pos) => write!(f,
                          "{}: {}", pos, msg),
            TransformationError::InvalidLanguageError(ref lang, ref msg, ref pos) =>
                write!(f, "{} [{}]: {}", pos, lang, msg),
            TransformationError::IoError(ref e, ref pos) => write!(f, "{}: {}", pos, e),
            TransformationError::JsonError(ref e, ref path) =>
                write!(f, "{}: {}", path, e),
            TransformationError::EncodingError(ref e, ref p) => write!(f, "{}: {}", e, p),
            TransformationError::XmlParserERrror(ref e, ref p) => write!(f, "{}: {}", e, p),
        }
    }
}

impl Error for TransformationError {
    fn description(&self) -> &str {
        match *self {
            TransformationError::ErrorneousStructure(_, _) => "invalid structure",
            TransformationError::EncodingError(_, _) => "invalid encoding",
            TransformationError::InvalidLanguageError(_, _, _) => "invalid language",
            TransformationError::IoError(ref err, _) => err.description(),
            TransformationError::JsonError(ref err, _) => err.description(),
            TransformationError::XmlParserERrror(ref err, _) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TransformationError::IoError(ref err, _) => err.cause(),
            TransformationError::JsonError(ref err, _) => err.cause(),
            TransformationError::XmlParserERrror(ref err, _) => err.cause(),
            _ => None,
        }
    }
}

/// allow seamless coercion from io::Error 
impl From<::std::io::Error> for TransformationError {
    fn from(err: ::std::io::Error) -> TransformationError {
        TransformationError::IoError(err, PositionType::None)
    }
}

/// allow hassle-free coercion from xml::reader::Error
impl From<::xml::reader::Error> for TransformationError {
    fn from(err: ::xml::reader::Error) -> TransformationError {
        TransformationError::XmlParserERrror(err, PositionType::None)
    }
}

impl From<::zip::result::ZipError> for TransformationError {
    fn from(err: ::zip::result::ZipError) -> TransformationError {
        use ::zip::result::ZipError;
        match err {
            ZipError::FileNotFound => TransformationError::IoError(io::Error::new(
                    io::ErrorKind::NotFound, "file not found"), PositionType::None),
            ZipError::Io(e) => TransformationError::IoError(e, PositionType::None),
            ZipError::InvalidArchive(ref msg) => TransformationError::ErrorneousStructure(
                    format!("Invalid zip file: {}", msg), PositionType::None),
            ZipError::UnsupportedArchive(ref msg) => TransformationError::ErrorneousStructure(
                    format!("Unsupported zip archive format: {}", msg),
                        PositionType::None),

        }
    }
}

/// Smallest unit of processing.
///
/// An entity is the smallest unit of processing. An entity can be an article, a
/// whole book or whatever seems feasible as a split point. It has two
/// characteristics: it holds the actual data and information about where the
/// entity came from.
pub struct Entity {
    pub content: String,
    pub position: PositionType,
}

impl Entity {
    pub fn with_path(c: String, p: PathBuf) -> Entity {
        Entity { content: c, position: PositionType::InDirectory(p) }
    }

    pub fn with_exact_pos(content: String, path: PathBuf, line: u64, col: u64)
            -> Entity {
        Entity { content, position: PositionType::InFile(path, line, col) }
    }

    /// Update the String content of the entity.
    pub fn update_content(&mut self, c: String) {
        self.content = c;
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
    fn preprocess(&self, input: &Entity) -> Result<Entity>;
}

