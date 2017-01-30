use std::path::{Path};
use super::input_source::*;
use super::common;

use pandoc;

pub struct Gutenberg;

impl GetIterator for Gutenberg {
    fn iter(&self, dst: &Path, _: Option<String>) ->
                Box<Iterator<Item=Result<String>>> {
        common::read_files(dst.into(), ".txt".into())
    }
}

impl Unformatter for Gutenberg {
    fn is_preprocessing_required(&self) -> bool {
        true
    }

    fn get_input_format(&self) -> pandoc::InputFormat {
        return pandoc::InputFormat::Markdown;
    }

    fn preprocess(&self, input: &str) -> Result<String> {
        let mut start = match input.find("*** START") {
            Some(e) => e,
            None => return Err(TransformationError::ErrorneousStructure(
                "no start delimiter found".into(), None),)
        };
        // find next line, because the "*** START" should be omitted  completely
        match input[start..].find("\n") {
            Some(pos) => start = start + pos,
            None => return Err(TransformationError::ErrorneousStructure(
                format!("no newline after the startdelimiter at position {}", start), None))
        };

        // get rid of the first 10 paragraphs by finding the beginning of the 11th
        // try to skip first 10 paragraphs
        if let Some(val) = skip_first_paragraphs(&input[start as usize..]) {
            start = start + val;
        }

        let end = find_end_of_book(&input[start as usize..])?;

        // some books contain arbitrari hypens, which often fill the gaps between two words:
        let output = input[start..(start + end)].replace("--", " ");
        Ok(output)
    }
}

/// After the marker that marks the beginning of a book, it might have "produced by", a table of
/// contents, a title page or other, useless data at the beginning. Since only proper text is
/// required for the processing, a fixed amount of paragraphs at the beginning is skipped.
fn skip_first_paragraphs(input: &str) -> Option<usize> {
    let mut skipped_paragraphs = 0;
    let mut start_pos = 0;
    while let Some(pos) = input[start_pos..].find("\n\n") {
        start_pos += pos + 2; // skip \n\n

        // if next character is != \n, it's a new paragraph
        if let Some(character) = input[start_pos..].chars().next() {
            if character != '\n' {
                skipped_paragraphs += 1;
            } // if it's \n, let it fall through and search for the next \n\n
        } else { // no \n\n found, unexpected end of document
            return None
        }
        if skipped_paragraphs >= 10 {
            return Some(start_pos);
        }
    }
    None
}


// find end of book, indicated by different markers
fn find_end_of_book(input: &str) -> Result<usize> {
    // this end-of-book is mandatory
    let mut end = input.find("*** END").ok_or_else(|| TransformationError::ErrorneousStructure(
            "no end delimiter found". into(), None))?;

    // some books have a stanza with the strings below. If one of these encountered, take this
    let new_end = if let Some(pos) = input.find("\nEnd of the Project Gutenberg") {
        Some(pos)
    } else if let Some(pos) = input.find("\nEnd of the project Gutenberg") {
        Some(pos)
    } else {
        None
    };
    if let Some(new_end) = new_end {
        if new_end < end {
            end = new_end;
        }
    }

    Ok(end)
}

