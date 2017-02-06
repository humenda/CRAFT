#[cfg(test)]
extern crate craft;

use craft::wikipedia::*;

fn preproc(input: &str) -> String {
    let preproc = MediawikiPreprocessor::new(input);
    preproc.preprocess().unwrap().to_string()
}

#[test]
fn test_that_preprocess_does_not_remove_unaffected_text() {
    assert_eq!(preproc("some text"), "some text");
}

#[test]
#[should_panic]
fn test_that_preproc_removes_blockquotes() {
    preproc("<blockquote>jo</blockquote>").find("<blockquote>").unwrap();
    preproc("<blockquote>jo</blockquote>").find("</blockquote>").unwrap();
}

#[test]
fn test_that_text_from_blockquote_is_preserved() {
    assert_eq!(preproc("<blockquote>text</blockquote>"), "text");
}

#[test]
fn test_that_text_around_blockquote_is_preserved() {
    assert_eq!(preproc("ab<blockquote></blockquote>cd"), "abcd");
    assert_eq!(preproc("ab<blockquote>cd</blockquote>ef"), "abcdef");
}

#[test]
fn test_that_blockquotes_with_arguments_are_removed() {
    assert_eq!(preproc("ab<blockquote useless='attribute'>cd</blockquote>ef"),
            "abcdef");
}

#[test]
fn test_that_text_within_ref_and_ref_itself_is_removed() {
    assert_eq!(preproc("Washington<ref>Capital</ref>."), "Washington.");
}


#[test]
fn test_that_unrelated_tags_are_not_affected() {
    let text = "<a href=\"fooo\">bar</a>";
    assert_eq!(preproc(text), text);
}

#[test]
fn test_that_straying_or_mathematical_lt_signs_work() {
    let text = "This is < an example";
    assert_eq!(preproc(text), text);
    let text = "Obviously, 4 < 5!";
    assert_eq!(preproc(text), text);
}

#[test]
fn test_that_tables_are_removed() {
    // I honestly don't know the difference between these two syntaxes, but they
    // both cause trouble with Pandoc
    assert_eq!(preproc("|{blabla\n|-invalid, but who cares\n...\n|}"), "");
    assert_eq!(preproc("{|blabla\n|-invalid, but who cares\n...\n}|"), "");
}

#[test]
fn test_that_text_bfore_and_after_tables_is_preserved() {
    assert_eq!(preproc("before\n|{blabla\n|-invalid, but who cares\n...\n|}after"),
            "before\nafter");
    assert_eq!(preproc("before\n{|blabla\n|-invalid, but who cares\n...\n}|after"),
    "before\nafter");
}

#[test]
fn test_random_vertical_bars_and_braces_dont_matter() {
    let text = "dskgj |{ dkjfkd\nk |}";
    assert_eq!(preproc(text), text);
    let text = "dskgj {| dkjfkd\nk }|";
    assert_eq!(preproc(text), text);
}

#[test]
fn test_real_world_string() {
    let text = "{{WP-hjælpesider}}\n\
    {| align=\"right\" style=\"background:transparent; border-bottom:1px 1px 0px 0px #a3b1bf solid;\"\n\
    |}\n";
    assert_eq!(preproc(text), "{{WP-hjælpesider}}\n\n");
}

