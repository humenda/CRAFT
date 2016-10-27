#[cfg(test)]
extern crate wikipedia2plain;

use wikipedia2plain::text2plain::*;

fn art2words(input: &str) -> String {
    article2words(input.to_string())
}

#[test]
fn test_all_words_preserved() {
    assert_eq!(art2words("test this"), "test this");
}

#[test]
fn test_other_whitespace_characters_are_ignored() {
    assert_eq!(art2words("\nok\t worked "), "ok worked");
}

#[test]
fn test_numbers_are_ignored() {
    assert_eq!(art2words("1990 was a special date"), "was a special date");
    assert_eq!(art2words("my 1st test"), "my test");
}

#[test]
fn test_that_punctuation_is_removed_and_words_preserved() {
    assert_eq!(art2words("However, I like it. :)"), "However I like it");
}

#[test]
fn test_words_with_hypen_work() {
    assert_eq!(art2words("this is a non-alcoholic drink"), "this is a \
                non-alcoholic drink");
    // do suspended hyphens work:
    assert_eq!(art2words("Using hard- and software"), "Using hard- and software");
}

#[test]
fn test_parenthesis_are_removed() {
    assert_eq!(art2words("(ignore that, ok?)"), "ignore that ok");
    assert_eq!(art2words("[ignore that, ok?]"), "ignore that ok");
    assert_eq!(art2words("{ignore that, ok?}"), "ignore that ok");
}

#[test]
fn test_that_apostrophies_may_be_contained_in_word() {
    assert_eq!(art2words("I'm not sure, O'raggley"), "I'm not sure O'raggley");
}

#[test]
fn test_semicolons_removed() {
    assert_eq!(art2words("ab; cd"), "ab cd");
    assert_eq!(art2words("ab;cd"), "");
}

#[test]
fn test_words_with_only_punctuation_etc_no_alphabetical_characters_removed() {
    assert_eq!(art2words("jo (''.) moo"), "jo moo");
}
