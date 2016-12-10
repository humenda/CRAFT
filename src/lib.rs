extern crate bzip2;
extern crate json;
#[macro_use]
extern crate log;
extern crate pandoc;
extern crate tempdir;
extern crate xml;


pub mod input_source; // must be first, Result<> defined here
#[macro_use]
mod common; // define this one second, contains macros
pub mod pandoc_executor;
pub mod text2plain;

/// input sources
pub mod europeana;
pub mod gutenberg;
pub mod wikipedia;

