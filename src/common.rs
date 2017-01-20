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

