/// This module contains helper functions, which are often used in the backends.

use htmlstream;
use std::ffi::OsString;
use std::fs;
use std::io::{Read};
use std::path;

use super::input_source::*;

/// as try!, but with options
macro_rules! get(
    ($e:expr) => (match $e {
        Some(e) => e,
        None => return None
    })
);

/// try! which works which returns a Option<Result<T>>
macro_rules! trysome(
    ($e:expr) => (
        match $e {
            Ok(d) => d,
            Err(e) => return Some(Err(From::from(e))),
        }
    )
);

/// try! which returns a Iterator<Result<_, TransformationError>> upon failure (useful fo r the
/// GetIter trait; this requires importing std::iter
macro_rules! tryiter(
    ($e:expr) => (
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

/// Consume a reader into a String
#[inline]
pub fn consume_reader(input: &mut Read) -> Result<String> {
    let mut consumed = String::new();
    input.read_to_string(& mut consumed)?;
    Ok(consumed)
}


/// Recurse files from given directory to list all files below the given path.
pub fn recurse_files(files_read: &mut Vec<String>, directory: &path::Path,
                     required_extension: &OsString) {
    let paths = fs::read_dir(directory).unwrap();
    for path in paths {
        if path.is_err() {
            warn!("couldn't list contents of {:?}", path);
            continue;
        }
        let path = path.unwrap().path();
        if path.is_dir() {
            recurse_files(files_read, &path, required_extension)
        } else { // is a file
            if path.extension().unwrap() == required_extension {
                let absolute_path = ::std::fs::canonicalize(path).unwrap();
                files_read.push(absolute_path.to_str().unwrap().into());
            } else { // error while converting path to str
                warn!("could not decode file name of {:?}", path);
            }
        }
    }
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

