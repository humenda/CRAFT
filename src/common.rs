/// This module contains helper functions, which are often used in the backends.

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


