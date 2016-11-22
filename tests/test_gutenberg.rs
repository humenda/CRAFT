#[cfg(test)]
extern crate wikipedia2plain;

use wikipedia2plain::gutenberg::*;
use wikipedia2plain::input_source::*;

fn preproc(input: &str) -> Result<String> {
    let g = Gutenberg;
    g.preprocess(input)
}


#[test]
fn test_beginning_and_end_are_detected() {
    let text = "*** START OF This ...\n\ncontent\n\n*** END ...";
    assert_eq!(preproc(text).unwrap(), "\n\ncontent\n\n");
}

#[test]
fn test_that_content_before_beginning_of_book_is_dropped() {
    let text = "\n\nbluuubl\n\nprologue\n\n*** START OF This ...\n\ncontent\n\n*** END ...";
    assert_eq!(preproc(text).unwrap(), "\n\ncontent\n\n");
}

#[test]
fn test_that_content_after_beginning_of_book_is_dropped() {
    let text = "*** START OF This ...\n\ncontent\n\n*** END ...\n\nFree pizza, if you'd like...";
    assert_eq!(preproc(text).unwrap(), "\n\ncontent\n\n");
}

#[test]
fn test_that_produced_by_is_stripped() {
    let text = "*** START OF This ...\n\nProduced by: Me! :)\n\ncontent\n\n*** END ...";
    assert_eq!(preproc(text).unwrap(), "\ncontent\n\n");
}

#[test]
#[should_panic]
fn test_that_unclosed_book_is_detected() {
    let text = "*** START OF\n...\n\n... invalid \n";
    preproc(text).unwrap();
}


