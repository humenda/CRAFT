use json::{self, object, JsonValue};
use pandoc;

use input_source::{Result, TransformationError};

static RETURN_ESCAPE_SEQUENCE: char = '\x07';
/// Manage conversion of documents with pandoc
///
/// This simple struct sets up a pandoc converter and adds the given format as an otpion. Its only
/// method `call_pandoc` transparently pipes the given String into pandoc and reads its output back
/// into a json String.
pub fn call_pandoc(input_format: pandoc::InputFormat, input: String) -> Result<String> {
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


/// Handle all different kind of pandoc objects in a Pandoc AST (e.g. Header or Str); for more doc,
/// see  stringify_text
fn handle_pandoc_entities(output: &mut String, entity: &mut object::Object) {
    // to mark the beginning of a new context for word2vec, newlines are required at certain points
    // (e.g. paragraphs); these are escaped with RETURN_ESCAPE_SEQUENCE and have to be surrounded
    // by spaces:
    let add_newline = |o: &mut String| {
        // add newline if there has been text inserted aaand previous text chunk is no newline
        // indicator; new line indicators are always " \x07 ", so the last char can be skipped
        if o.len() > 1 && o.chars().last().unwrap() == RETURN_ESCAPE_SEQUENCE {
            o.push(' ');
            o.push(RETURN_ESCAPE_SEQUENCE);
            o.push(' ');
        }
    };

    // every pandoc object consists of a "t" (type) and a content "c"; match on the type:
    match entity.get("t").unwrap_or_else(|| panic!("broken json")).to_string().as_ref() {
        // add a space, if last character wasn't already a space
        "Space" | "LineBreak" | "SoftBreak" => match output.chars().rev().next() {
            Some(x) if !x.is_whitespace() => output.push(' '),
            _ => ()
        },

        // use take_string to extract the string of this element
        "Str" => if let Some(x) = entity.get_mut("c") {
            // ToDo: shorter?
            output.push_str(x.take_string().unwrap().clone().as_ref());
        },

        // handle heading; third element  contains content
        "Header" => if let Some(heading) = entity.get_mut("c") {
            recurse_json_tree(output, &mut heading[2]); // 2nd element of array contains content
            add_newline(output);
        },

        // these should have a newline after these elements
        "Para" | "Plain" | "BlockQuote" | "BulletList" | "DefinitionList" =>
            if let Some(thing) = entity.get_mut("c") {
                recurse_json_tree(output, thing); // recurse list of children
                add_newline(output);
        },

        // these elements also contain a list of children, but should not be followed by a newline
        "Emph" | "Strong" | "Strikeout" | "SmallCaps" | "Note" =>
            if let Some(thing) = entity.get_mut("c") {
                recurse_json_tree(output, thing);
        },

        // these are arrays with two items, where the first are attributes
        t @ "OrderedList" | t @ "Div" | t @ "Span" =>
            if let Some(array) = entity.get_mut("c") {
                match *array {
                    JsonValue::Array(ref mut x) if x.len() == 2 => {
                        recurse_json_tree(output, &mut x[1]);
                        if t != "Span" { // newline after all except span
                            add_newline(output);
                        }
                    },
                    _ => panic!("{}: expected a JSON array with length 2, got: {}",
                                t, array),
            }
        },

        // these have a JsonValue::Array with thee values, where the second is content
        y @ "Link" | y @ "Image" => handle_external_references(output, y, entity),

        // types to ignore
        "CodeBlock" | "RawBlock" | "HorizontalRule" | "Table" | "Superscript" |
                "Subscript" | "Cite" | "Code" | "Math" | "RawInline" | "Null" => (),
        _ => panic!("Unknown type pandoc AST identifier found: {:?}", entity),
    }
}

/// handle links and images entities separately
///
/// For links, it only makes sense to preserve the displayed text. For images it depends on the
/// image description. Logos and icons often only consist of 1-3 words, so they do not represent
/// valuable contextual information. Therefore only longer image descriptions are kept.
fn handle_external_references(output: &mut String, id: &str, entity: &mut object::Object) {
    let thing = entity.get_mut("c");
    if !thing.is_some() {
        return;
    }
    let thing = thing.unwrap();
    // text found within the link / image
    let mut read_text = String::new();
    match *thing {
        JsonValue::Array(ref mut x) if x.len() == 3 =>
            recurse_json_tree(&mut read_text, &mut x[1]),
        _ => panic!("{}: expected a JSON array, got: {}", id, thing),
    }
    // treat link and image differently:
    match id {
        "Link" => output.push_str(&read_text), // add as parsed
        "Image" => { // count number of words and only keep if > 3 in image description
            let mut split = read_text.split_whitespace();
            let mut words_found = 0;
            while let Some(_) = split.next() {
                if words_found > 3 {
                    break;
                }
                words_found += 1;
            }
            // if there's a proper description, keep text, otherwise discard
            if words_found >= 3 {
                output.push_str(&read_text);
            }
        },
        _ => panic!("unreachable code!")
    }
}

/// recursively extract all string parts; more doc at the public function stringify_text
fn recurse_json_tree(output: &mut String, jsval: &mut JsonValue) {
    match jsval {
        &mut JsonValue::Null => return,
        &mut JsonValue::Short(data) => output.push_str(data.as_str().into()),
        &mut JsonValue::String(ref mut data) => output.push_str(data.as_str().into()),
        &mut JsonValue::Number(data) => output.push_str(data.to_string().as_str()),
        &mut JsonValue::Boolean(data) => output.push_str(data.to_string().as_str()),
        &mut JsonValue::Object(ref mut entity) => handle_pandoc_entities(output, entity),
        &mut JsonValue::Array(ref mut values) => {
            let lastindex = match values.len() {
                x if x > 0 => x - 1,
                _ => 0
            };
            for (i, mut val) in values.iter_mut().enumerate() {
                recurse_json_tree(output, &mut val);
                // between the items of an array are sometimes no spaces (e.g. in lists), so check and
                // insert a space
                match output.chars().rev().next() {
                    Some(x) if !x.is_whitespace() && i < lastindex =>
                        output.push(' '),
                    _ => ()
                };
            };
        }
    }
}

/// Filter the Pandoc AST for plain text
///
/// Pandoc parses the document into an abstract syntax tree (AST), which can represent the whole
/// document. Objects (JSON) always consist of `"t":"some_type"` and 
/// `"c":`content`. Content can either be another object or most likely, a JSON array. This module
/// parses the plain text bits out of this AST. It does not preserve white space, but puts each
/// word, separated by a space, next to each other. Hence a String with a single line is the
/// result.
pub fn stringify_text(pandoc_dump: String) -> String {
    let ast = json::parse(&pandoc_dump).unwrap();
    let mut output = String::new();
    match ast {
        JsonValue::Array(mut values) => if values.len() == 2 {
            recurse_json_tree(&mut output, &mut values[1]);
        },
        _ => panic!("expected JSON document with an Array at top level object \
                    and two entries: unmeta and the contents of the parsed document.")
    };
    output
}


////////////////////////////////////////////////////////////////////////////////
// strip punctuation

// Test whether given character is a enclosing character like quotes or
// parenthesis.
fn is_enclosing_character(c: char) -> bool {
    match c {
        '(' | ')' | '[' | ']' | '{' | '}' | '"' | '„' | '”' | '“' | '‚' 
            | '’' | '‘' | '«' | '»'  => true,
        _ => false,
    }
}

fn is_punctuation(c: char) -> bool {
    match c {
        '.' | ',' | ':' | ';' | '?' | '!' | '…' | '–' => true,
        _ => false,
    }
}

// test whether character is some kind of apostrophe (and similar)
fn is_apostrophe(c: char) -> bool {
    match c {
        '\'' | '`' | '‚' | '‘' | '’' => true,
        _ => false,
    }
}



// Test whether all characters are alphabetical; could be a closure, but early
// return might make it SLIGHTLY more efficient; "-" is part  valid within a
// word, too
// Note: apostrophes count as alphabetical, too.
fn all_chars_alphabetical(word: &String) -> bool {
    // make sure that a word not only consists of dashes:
    let mut found_one_alphabetical_character = false;
    for character in word.chars() {
        if character.is_alphabetic() {
            if !found_one_alphabetical_character {
                found_one_alphabetical_character = true;
            }
        } else if is_apostrophe(character) {
            // works in a word, but do not set found_one_alphabetical_character :)
        } else if character != '-' { //not alphabetical and no dash
            return false; // early return, non-alphabetical character
        }
    }
    found_one_alphabetical_character
}

// remove parenthesis and similar from word
fn remove_enclosing_characters(input: &mut String) {
    while let Some(x) = input.chars().rev().next() {
        let _ = match is_enclosing_character(x) {
            true => {
                input.pop();
            },
            false => break,
        };
    }

    // remove enclosing characters at beginning
    while let Some(y) = input.chars().next() {
        let _ = match is_enclosing_character(y) {
            true => input.remove(0),
            false => break
        };
    }
}

/// strip punctuation and return whether punctuation has been stripped
fn remove_punctuation(input: &mut String) {
    while let Some(punct) = input.chars().rev().next() {
        let _ = match is_punctuation(punct) {
            true => input.pop(),
            false => break
        };
    }
}

// only keep the words of a text, separated by spaces. Line breaks, indentation, multiple spaces
// and punctuation (including parenthesis, etc.) are removed.
pub fn text2words(input: String) -> String {
    let mut words = String::new();

    for word in input.split_whitespace() {
        // newline indicators are " \x07 ", so length 3 and word[1] === RETURN_ESCAPE_SEQUENCE:
        if word.len() == 1 && word.chars().next() == Some(RETURN_ESCAPE_SEQUENCE) {
            words.push('\n');
        } else {
            // remove punctuation, then  enclosing characters (quotations or parenthesis) and then
            // remove cpunctuation again
            let mut word = String::from(word);
            //  remove_punctuation yields boolean telling whether punctuation was removed; Useful for determining sentences. Currently unused.
            remove_punctuation(&mut word);
            remove_enclosing_characters(&mut word);
            remove_punctuation(&mut word);
            if all_chars_alphabetical(&word) {
                if words.len() != 0 && words.chars().last() != Some('\n') {
                    words.push(' ');
                }

                words.push_str(word.to_lowercase().as_str());
            }
        }
    }
    words
}

