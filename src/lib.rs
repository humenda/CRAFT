extern crate bzip2;
extern crate json;
#[macro_use]
extern crate log;
extern crate pandoc;
extern crate xml;


pub mod input_source; // must be first, Result<> defined here
#[macro_use]
mod common; // define this one second, contains macros
pub mod textfilter;

/// input sources
pub mod codecivil;
pub mod europeana;
pub mod gutenberg;
pub mod wikipedia;

