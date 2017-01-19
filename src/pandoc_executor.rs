//! This module provides a function to call Pandoc.
//!
//" This module contains a type which controls the execution of Pandoc.
use pandoc;
use std::path;

use input_source::*;

/// Manage conversion of documents with pandoc
///
/// This simple struct sets up a pandoc converter and adds the given format as an otpion. Its only
/// method `call_pandoc` transparently pipes the given String into pandoc and reads its output back
/// into a json String.
pub fn call_pandoc(input_format: pandoc::InputFormat, input: String)
    -> Result<String> {
    let mut p = pandoc::new();
    p.set_output_format(pandoc::OutputFormat::Json);
    //p.set_input_format(input_format.clone());
    p.set_input_format(input_format);
    p.set_output(pandoc::OutputKind::Pipe);
    p.set_input(pandoc::InputKind::Pipe(input.clone()));
    match p.execute() {
        Ok(pandoc::PandocOutput::ToBuffer(data)) => Ok(data),
        Ok(_) => panic!(format!("Expected converted data, got file name instead\nThis is a bug and needs to be fixed before continuing.")),
        Err(x) => Err(TransformationError::ErrorneousStructure(format!("{:?}\nArticle:\n{}\n",
                                                                           x, input), None))
    }
}

