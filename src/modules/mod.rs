//! This module contains all input sources of CRAFT. An input source has to provide an iterator of
//! type `Iterator<Item=Result<String>>`, `use input_source::Result;`. It also may implement the
//! `UnFormatter` trait, which defines preprocessing functionality for problematic input data which
//! leads to pandoc crashes.

pub mod codecivil;
pub mod dgt;
pub mod europeana;
pub mod gutenberg;
pub mod wikipedia;
