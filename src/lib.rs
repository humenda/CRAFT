//! CRAFT — CorpoRA-based Freedict Token extractor
//!
//! This library provides modules and tokenization helpers to preprocess text data for neuronal
//! networks, more precisely for Word2vec. While it has been developed to work for the thesaurus
//! generator **Alt**, it can be used for any text processing purposes.
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
pub mod modules;

