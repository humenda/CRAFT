

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
            true => input.pop(),
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

fn remove_punctuation(input: &mut String) {
    while let Some(punct) = input.chars().rev().next() {
        let _ = match is_punctuation(punct) {
            true => input.pop(),
            false => break
        };
    }
}

pub fn article2words(input: String) -> String {
    let mut words = String::new();

    for word in input.split_whitespace() {
        let mut word = String::from(word);
        remove_enclosing_characters(&mut word);
        remove_punctuation(&mut word);
        if all_chars_alphabetical(&word) {
            if words.len() != 0 {
                words.push(' ');
            }
            words.push_str(word.as_str());
        }
    }
    words
}

