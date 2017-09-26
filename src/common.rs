//! Common bits for modules and the CRAFT implementation.
//!
//! This module contains common functions, macros and iterators used by both modules and the CRAFT
//! implementation.

use htmlstream;
use std::ffi::OsString;
use std::fs;
use std::io::{Read};
use std::path;

use super::input_source::*;

/// Option handling for iterators
/// 
/// This macro works exactly like `try!`, but on `Option`.
macro_rules! get(
    ($e:expr) => (match $e {
        Some(e) => e,
        None => return None
    })
);

/// Try for iterators of type `Item=Result<_>`
///
/// For an iterator to be able to transport an error, a `Result` is encapsulated into the `Option`
/// type. Handling errors in such an iterator can be tedious, because in the case of an error,
/// `Some(Err(…))` has to be returned. This Macro will unwrap an `Ok(_)` and `return Some(Err…))`
/// otherwise.
macro_rules! trysome(
    ($e:expr) => (
        match $e {
            Ok(d) => d,
            Err(e) => return Some(Err(From::from(e))),
        }
    )
);

/// A `try!`-alike macro returning an iterator upon failure.
/// Input modules are required to provide an iterator providing text chunks (articles, books,
/// etc.). Because of the internal implementation, the only way to propagate an error from an
/// iterator is to return an error as the first element and `None` afterwards. This macro unwraps a
/// result and if an error is found, a one-item iterator is returned with the error wrapped.
/// GetIter trait; this requires importing std::iter
macro_rules! tryiter(
    ($e:expr) => (
        use std::iter;
        match $e {
            Ok(d) => d,
            Err(e) => return iter::once(Err(From::from(e))),
        }
    )
);


/// Read the contents of a file to a String.
pub fn read_file(path: &path::Path) -> Result<String> {
    let f = fs::File::open(path);
    let mut content = String::new();
    f?.read_to_string(&mut content)?;
    Ok(content)
}

/// Extract links from given HTML document
///
/// This function parses the given document for `<a/>` tags and saves all their href attributes, so
/// all URLs, into a vector.
pub fn extract_links(document: &str) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    for (_, tag) in htmlstream::tag_iter(document) {
        if tag.name == "a" {
            // add hreffrom a tag
            links.extend(htmlstream::attr_iter(&tag.attributes).filter(|attr|
                    attr.1.name == "href").map(|x| x.1.value));
        }
    }
    links
}


/// Emit filtered file paths
///
/// This iterator allows for recursive iteration over a file tree, while
/// automatically filtering for a required file extension.
pub struct Files {
    file_list: fs::ReadDir,
    requested_file_ending: OsString
}

impl Files {
    /// Return a new file iterator.
    pub fn new<P>(path: P, extension: OsString) -> Result<Files>
            where P: AsRef<path::Path> {
        Ok(Files {
            file_list: fs::read_dir(path)?,
            requested_file_ending: extension
        })
    }
}

impl Iterator for Files {
    type Item = Result<path::PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(fpath) = self.file_list.next() {
            let fpath = trysome!(fpath.map(|x|
                    x.path()).map_err(|e| TransformationError::IoError(e,
                           PositionType::None)));
            if fpath.extension() == Some(&self.requested_file_ending) {
                return Some(Ok(fpath))
            }
        }
        None
    }
}


/// Return content of a file found below given path
///
/// This function creates an iterator which recurses all files in a given
/// directory with a given extension and returns its content.
pub fn read_files(input: path::PathBuf, extension: OsString)
            -> Box<Iterator<Item=Result<Entity>>> {
    Box::new(Files::new(input, extension).unwrap().map(|x| match x {
        Ok(fpath) => match fs::File::open(&fpath) {
            Ok(mut r) => { // read file to String
                let mut res = String::new();
                r.read_to_string(&mut res)?;
                Ok(Entity::with_path(res, fpath))
            },
            Err(e) => Err(TransformationError::IoError(e, PositionType::InDirectory(fpath))),
        },
        Err(e) => Err(e),
    }))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_that_all_links_recognized() {
        let html = r#"<p><a href="foo"></a></p><a href="bar"></a></p>"#;
        let links = extract_links(html);
        links.iter().find(|x: &&String| &x[..] == "foo").unwrap();
        links.iter().find(|x: &&String| &x[..] == "bar").unwrap();
    }

    #[test]
    fn test_no_false_links_are_detected() {
        let html = r#"<p><a href="foo"></a></p><p href="bar"></p></p>"#;
        let links = extract_links(html);
        assert_eq!(links.len(), 1);
    }
}

