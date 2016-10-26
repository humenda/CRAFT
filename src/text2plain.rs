// ToDo: use &str for word iterator to prevent creation of strings, but how?
use std::str::Chars;

struct WordIterator<'a> {
    characters: Chars<'a>,
    last_parsed_word: String,
}

impl<'a> WordIterator<'a> {
    pub fn new(input: &'a String) -> WordIterator<'a> {
        WordIterator { characters: input.chars(),
                        last_parsed_word: String::new() }
    }

    fn fetch_next_word(&mut self) -> Option<String> {
        while let Some(cur) = self.characters.next() {
            if cur.is_whitespace() {
                if self.last_parsed_word.len() > 0 {
                    let ret = self.last_parsed_word.clone();
                    self.last_parsed_word.clear();
                    return Some(ret);
                }
                // no else, ignore subsequent white space
            } else {
                self.last_parsed_word.push(cur);
            }
        }
        if self.last_parsed_word.len() > 0 {
            let ret = Some(self.last_parsed_word.clone());
            self.last_parsed_word.clear();
            return ret;
        }
        None
    }
}

impl<'a> Iterator for WordIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.fetch_next_word()
    }
}

////////////////////////////////////////////////////////////////////////////////
// strip punctuation and enclosing  characters
////////////////////////////////////////////////////////////////////////////////

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

// test wehther character is some kind of apostrophe (and similar)
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
        if character.is_alphabetic() || is_apostrophe(character) {
            if !found_one_alphabetical_character {
                found_one_alphabetical_character = true;
            }
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

    for word in WordIterator::new(&input) {
        let mut word = word.clone(); // ToDo: implement trait to support either iter_mut or even better into_iter vor by-value reference
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

