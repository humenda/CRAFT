
use super::super::input_source::{Entity, PositionType, Result, TransformationError, Unformatter};
use pandoc;

pub struct Gutenberg;


impl Unformatter for Gutenberg {
    fn is_preprocessing_required(&self) -> bool {
        true
    }

    fn get_input_format(&self) -> pandoc::InputFormat {
        return pandoc::InputFormat::Markdown;
    }

    fn preprocess(&self, input: &Entity) -> Result<Entity> {
        let content = &input.content;
        let mut start = match content.find("*** START") {
            Some(e) => e,
            None => return Err(TransformationError::ErrorneousStructure(
                "no start delimiter found".into(), input.position.clone()))
        };
        // find next line, because the "*** START" should be omitted  completely
        match content[start..].find("\n") {
            Some(pos) => start = start + pos,
            None => return Err(TransformationError::ErrorneousStructure(
                format!("no newline after the startdelimiter at position {}",
                        start), input.position.clone()))
        };

        // get rid of the first 10 paragraphs by finding the beginning of the 11th
        // try to skip first 10 paragraphs
        if let Some(val) = skip_first_paragraphs(&content[start as usize..]) {
            start = start + val;
        }

        let end = find_end_of_book(&content[start as usize..]).map_err(|mut e| {
            e.inject_position(input.position.clone());e
        })?;

        // some books contain arbitrari hypens, which often fill the gaps between two words:

        Ok(Entity { content: content[start..(start + end)].replace("--", " "),
            position: input.position.clone() })
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
fn find_end_of_book(content: &str) -> Result<usize> {
    // this end-of-book is mandatory
    let mut end = content.find("*** END").ok_or_else(|| TransformationError::ErrorneousStructure(
            "no end delimiter found". into(), PositionType::None))?;

    // some books have a stanza with the strings below. If one of these encountered, take this
    let new_end = if let Some(pos) = content.find("\nEnd of the Project Gutenberg") {
        Some(pos)
    } else if let Some(pos) = content.find("\nEnd of the project Gutenberg") {
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

