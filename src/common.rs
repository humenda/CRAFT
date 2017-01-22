/// This module contains helper functions, which are often used in the backends.

use htmlstream;
use std::fs;
use std::io::Read;
use std::path;

use super::input_source::*;

macro_rules! get(
    ($e:expr) => (match $e {
        Some(e) => e,
        None => return None
    })
);

/// Return the contents of a file
pub fn read_file_from_str(path: &str) -> Result<String> {
    let mut content = String::new();
    let mut f = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(TransformationError::IoError(e, Some(String::from(path)))),
    };
    try!(f.read_to_string(&mut content));
    Ok(content)
}

/// Return the contents of a file
pub fn read_file(path: &path::Path) -> Result<String> {
    let f = fs::File::open(path);
    let mut content = String::new();
    f?.read_to_string(&mut content)?;
    Ok(content)
}

// recursively gather all files in a given directory with a given file extension
pub fn recurse_files(files_read: &mut Vec<String>, directory: &path::Path,
                     required_extension: &str) {
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
            if let Some(fname) = path.to_str().clone() {
                if fname.ends_with(required_extension) {
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

/// Emit all files of a directory which match a given ending
pub struct Files {
    file_list: fs::ReadDir,
    requested_file_ending: String
}

impl Files {
    /// Return a new file iterator.
    pub fn new(path: &path::Path, ending: &str) -> Result<Files> {
        Ok(Files {
            file_list: path.read_dir()?,
            requested_file_ending: ending.into()
        })
    }
}

impl Iterator for Files {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Result<String>> {
        while let Some(file) = self.file_list.next() {
            match file {
                Ok(e) => {
                    let fname = e.file_name();
                    let fname = get!(fname.to_str());
                    if fname.ends_with(&self.requested_file_ending)  {
                        return match read_file(&e.path()) {
                            Ok(x) => return Some(Ok(x)),
                            Err(e) => Some(Err(e)),
                        }
                    }
                },
                Err(e) => return Some(Err(TransformationError::IoError(e, None))),
            }
        }
        None
    }
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

