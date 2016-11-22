use json;
use json::*;

/// Handle all different kind of pandoc objects in a Pandoc AST (e.g. Header or Str); for more doc,
/// see  stringify_text
fn handle_pandoc_entities(output: &mut String, entity: &mut object::Object) {
    // every pandoc object consists of a "t" (type) and a content "c"; match on the type:
    match entity.get("t").unwrap_or_else(|| panic!("broken json")).to_string().as_ref() {
        // add a space, if last character wasn't already a space
        "Space" | "LineBreak" | "SoftBreak" => match output.chars().rev().next() {
            Some(x) if !x.is_whitespace() => output.push(' '),
            _ => ()
        },

        // use take_string to extract the string of this element
        "Str" => if let Some(x) = entity.get_mut("c") {
            output.push_str(x.take_string().unwrap().clone().as_ref());
        },

        // handle heading; third element  contains content
        "Header" => if let Some(heading) = entity.get_mut("c") {
            recurse_json_tree(output, &mut heading[2]); // 2nd element of array contains content
        },

        // these all contain JsonValue::Array, so better process them with recursion
        "Para" | "Plain" | "BlockQuote" | "BulletList" | "DefinitionList" |
                "Emph" | "Strong" | "Strikeout" | "SmallCaps" | "Note" =>
            if let Some(thing) = entity.get_mut("c") {
                recurse_json_tree(output, thing);
        },

        // these are arrays with two items, where the first are attributes
        t @ "OrderedList" | t @ "Div" | t @ "Span" =>
            if let Some(array) = entity.get_mut("c") {
                match *array {
                    JsonValue::Array(ref mut x) if x.len() == 2 =>
                        recurse_json_tree(output, &mut x[1]),
                    _ => panic!("{}: expected a JSON array with length 2, got: {}",
                                t, array),
            }
        },

        // these have a JsonValue::Array with thee values, where the second is content
        y @ "Link" | y @ "Image" => if let Some(thing) = entity.get_mut("c") {
            match *thing {
                JsonValue::Array(ref mut x) if x.len() == 3 =>
                    recurse_json_tree(output, &mut x[1]),
                _ => panic!("{}: expected a JSON array, got: {}", y, thing),
            }
        },

        // types to ignore
        "CodeBlock" | "RawBlock" | "HorizontalRule" | "Table" | "Superscript" |
                "Subscript" | "Cite" | "Code" | "Math" | "RawInline" | "Null" => (),
        _ => panic!("Unknown type pandoc AST identifier found: {:?}", entity),
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
fn remove_punctuation(input: &mut String) -> bool {
    let old_length = input.len();
    while let Some(punct) = input.chars().rev().next() {
        let _ = match is_punctuation(punct) {
            true => input.pop(),
            false => break
        };
    }
    old_length == input.len() // if they don't match, punctuation has been removed
}

// only keep the words of a text, separated by spaces. Line breaks, indentation, multiple spaces
// and punctuation (including parenthesis, etc.) are removed.
pub fn text2words(input: String) -> String {
    let mut words = String::new();

    for word in input.split_whitespace() {
        // remove punctuation, then  enclosing characters (quotations or parenthesis) and then
        // remove cpunctuation again
        let mut word = String::from(word);
        let mut punct_was_removed = remove_punctuation(&mut word);
        remove_enclosing_characters(&mut word);
        punct_was_removed = remove_punctuation(&mut word) || punct_was_removed;
        if all_chars_alphabetical(&word) {
            if words.len() != 0 {
                words.push(' ');
            }
            words.push_str(word.to_lowercase().as_str());
        }
    }
    words
}

