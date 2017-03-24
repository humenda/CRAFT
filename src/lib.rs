extern crate bzip2;
extern crate isolang;
extern crate json;
extern crate htmlstream;
#[macro_use]
extern crate log;
extern crate pandoc;
extern crate xml;
extern crate zip;


pub mod input_source; // must be first, Result<> defined here
#[macro_use]
pub mod common; // define this one second, contains macros
pub mod textfilter;

/// input sources
pub mod codecivil;
pub mod dgt;
pub mod europeana;
pub mod gutenberg;
pub mod wikipedia;

