#[cfg(test)]
extern crate craft;

use craft::gutenberg::*;
use craft::input_source::*;

fn preproc(input: &str) -> Result<String> {
    let g = Gutenberg;
    g.preprocess(input)
}

static TEN_PARS: &'static str = "\n\ntest test test\n\ntest test test\n\ntest test test\n\ntest test test\n\ntest test test\n\ntest test test\n\ntest test test\n\ntest test test\n\ntest test test\n\ntest test test";

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
fn test_that_first_paragraphs_stripped() {
    let text = format!("*** START OF This ...\n{}\n\ncontent\n*** END", TEN_PARS);
    assert_eq!(preproc(&text).unwrap(), "content\n");
}

#[test]
#[should_panic]
fn test_that_unclosed_book_is_detected() {
    let text = "*** START OF\n...\n\n... invalid \n";
    preproc(text).unwrap();
}

#[test]
fn test_that_useless_hyphens_are_removed() {
    let text = "*** START OF book\n\
                \n--Foo bar--dummy, value\n\
                *** END OF this book\n";
    assert_eq!(preproc(text).unwrap(), "\n\n Foo bar dummy, value\n");
}

