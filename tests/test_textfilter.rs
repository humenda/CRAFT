#[cfg(test)]
extern crate craft;

use craft::*;
use craft::textfilter::*;

fn art2words(input: &str) -> String {
    text2words(input.to_string())
}

#[test]
fn test_all_words_preserved() {
    assert_eq!(art2words("test this"), "test this\n");
}

#[test]
fn test_other_whitespace_characters_are_ignored() {
    assert_eq!(art2words("\nok\t worked "), "ok worked\n");
}

#[test]
fn test_numbers_are_ignored() {
    assert_eq!(art2words("1990 was a special date"), "was a special date\n");
    assert_eq!(art2words("my 1st test"), "my test\n");
}

#[test]
fn test_that_punctuation_is_removed_and_words_preserved() {
    assert_eq!(art2words("However, I like it. :)"), "however i like it\n");
}

#[test]
fn test_words_with_hyphen_work() {
    assert_eq!(art2words("this is a non-alcoholic drink"), "this is a \
                non-alcoholic drink\n");
    // do suspended hyphens work:
    assert_eq!(art2words("Using hard- and software"), "using hard- and software\n");
}

#[test]
fn test_parenthesis_are_removed() {
    assert_eq!(art2words("(ignore that, ok?)"), "ignore that ok\n");
    assert_eq!(art2words("[ignore that, ok?]"), "ignore that ok\n");
    assert_eq!(art2words("{ignore that, ok?}"), "ignore that ok\n");
}

#[test]
fn test_that_apostrophies_may_be_contained_in_word() {
    assert_eq!(art2words("I'm not sure, O'raggley"), "i'm not sure o'raggley\n");
}

#[test]
fn test_semicolons_removed() {
    assert_eq!(art2words("ab; cd"), "ab cd\n");
    assert_eq!(art2words("ab;cd"), "");
}

#[test]
fn test_words_with_only_punctuation_etc_no_alphabetical_characters_removed() {
    assert_eq!(art2words("jo (''.) moo"), "jo moo\n");
}

#[test]
fn test_that_unicode_quotes_are_removed() {
    // example from the real world
    let text = "Deutsch „die Hauptstadt“.";
    assert_eq!(art2words(text), "deutsch die hauptstadt\n");
}

#[test]
fn test_newline_markers_keep_newline() {
    let text = "abc \x07 def".into();
    assert_eq!(art2words(text), "abc\ndef\n");
}

////////////////////////////////////////////////////////////////////////////////
// test the JSON AST filter

// this function calls the JSON2text function and replaces all " \u{7}" sequuuences through \n;
// this is partly what textfilter::text2words does, but here it's solely for easier testability
fn call_filter(js_str: String) -> String {
    let result = textfilter::stringify_text(js_str).unwrap();
    let result = result.replace(&format!(" {} ", textfilter::RETURN_ESCAPE_SEQUENCE), "\n");
    result
}

////////////////////////////////////////////////////////////////////////////////
// test inline elements

#[test]
fn test_that_str_is_serialized_correctly() {
    // this document contains a Str element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Str\", \"c\": \"simplestr\"}]}]]".into(); 
    assert_eq!(call_filter(json_str), "simplestr\n");
}


#[test]
fn test_that_emph_is_serialized_correctly() {
    // this document contains a Emph element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Emph\", \"c\": [{\"t\": \"Str\", \"c\": \"emphasized\"}]}]}]]".into();
    assert_eq!(call_filter(json_str), "emphasized\n");
}


#[test]
fn test_that_strong_is_serialized_correctly() {
    // this document contains a Strong element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": [{\"c\": \"ok\", \"t\": \"Str\"}],\
       \"t\": \"Strong\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "ok\n");
}


#[test]
fn test_that_strikeout_is_serialized_correctly() {
    // this document contains a Strikeout element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Str\", \"c\": \"let\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"it\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"be\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Strikeout\", \"c\": [{\"t\": \"Str\", \"c\": \"deleted.\"}]}]}]]".into();
    assert_eq!(call_filter(json_str), "let it be deleted.\n");
}


#[test]
fn test_that_superscript_is_serialized_correctly() {
    // this document contains a Superscript element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Str\", \"c\": \"2\"},\
       {\"t\": \"Superscript\", \"c\": [{\"t\": \"Str\", \"c\": \"1024\"}]},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"is\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"pretty\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"big\"}]}]]".into();
    assert_eq!(call_filter(json_str), "2 is pretty big\n");
}


#[test]
fn test_that_subscript_is_serialized_correctly() {
    // this document contains a Subscript element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Str\", \"c\": \"drink\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"enough\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"H\"},\
       {\"t\": \"Subscript\", \"c\": [{\"t\": \"Str\", \"c\": \"2\"}]},\
       {\"t\": \"Str\", \"c\": \"O\"}]}]]".into();
    assert!(call_filter(json_str).starts_with("drink enough H O"));
}


#[test]
fn test_that_smallcaps_is_serialized_correctly() {
    // this document contains a Smallcaps element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"SmallCaps\", \"c\": [{\"t\": \"Str\", \"c\": \"UNO\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"IEEE\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"FOO\"}]}]}]]".into();
    assert_eq!(call_filter(json_str), "UNO IEEE FOO\n");
}


#[test]
fn test_that_cite_is_serialized_correctly() {
    // this document contains a Cite element, which should be ignored
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": [[{\"citationPrefix\": [], \"citationId\": \"someauthor\", \"citationNoteNum\": 0, \"citationMode\": {\"c\": [], \"t\": \"NormalCitation\"},\
       \"citationHash\": 0, \"citationSuffix\": []}],\
       [{\"c\": \"[@someauthor]\", \"t\": \"Str\"}]], \"t\": \"Cite\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "");
}


#[test]
fn test_that_code_is_serialized_correctly() {
    // this document contains a Code, which should be ignored element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": \"a\", \"t\": \"Str\"},\
       {\"c\": [[\"\", [], []], \"b\"], \"t\": \"Code\"},\
       {\"c\": \"c\", \"t\": \"Str\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "a c\n");
}


#[test]
fn test_that_space_is_serialized_correctly() {
    // this document contains a Space element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": \"a\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"b\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"c\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"d\", \"t\": \"Str\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "a b c d\n");
}


#[test]
fn test_that_linebreak_is_serialized_correctly() {
      // this document contains a LineBreak element, which is ignored
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": \"here\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"LineBreak\"},\
       {\"c\": \"is\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"a\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"newline\", \"t\": \"Str\"}],\
       \"t\": \"Para\"}]]".into();
    assert!(call_filter(json_str).starts_with("here is a newline"));
}


#[test]
fn test_that_math_is_ignored() {
    // this document contains a maths environments, both should be ignored
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Math\", \"c\": [{\"t\": \"InlineMath\", \"c\": []},\
       \"a\"]},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"and\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Math\", \"c\": [{\"t\": \"DisplayMath\", \"c\": []},\
       \"b\"]}]}]]".into();
    assert_eq!(textfilter::text2words(call_filter(json_str)), "and\n");
}


#[test]
fn test_that_rawinline_is_serialized_correctly() {
    // this document contains a RawInline element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"RawInline\", \"c\": [[\"\", [], []], \"ignore me\"]}]}]]".into();
    assert_eq!(call_filter(json_str), "");
}


#[test]
fn test_that_only_linktext_is_kept() {
    // this document contains a Link element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": [[\"\", [], []], [{\"c\": \"this\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"is\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"ok\", \"t\": \"Str\"}],\
       [\"and%20this%20isn't\", \"fig:\"]], \"t\": \"Image\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "this is ok\n");
}


#[test]
fn test_that_image_alt_text_is_kept() {
    // this document contains a Image element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": [[\"\", [], []], [{\"c\": \"an\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"image\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"of\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"a\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"beef\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"steak\", \"t\": \"Str\"}],\
       [\"meeeeeeeet.png\", \"fig:\"]], \"t\": \"Image\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "an image of a beef steak\n");
}

#[test]
fn test_that_images_with_short_descriptions_are_ignored() {
    // this document contains a Image element
    let json_str: String = "[{\"unMeta\":{}},\
      [{\"t\":\"Para\",\"c\":[{\"t\":\"Str\",\"c\":\"bla\"},\
      {\"t\":\"SoftBreak\",\"c\":[]},\
      {\"t\":\"Image\",\"c\":[[\"\",[],[]],[{\"t\":\"Str\",\"c\":\"infobox\"}],\
      [\"bla.png\",\"\"]]},\
      {\"t\":\"SoftBreak\",\"c\":[]},\
      {\"t\":\"Str\",\"c\":\"cow\"}]}]]\
      ".into();
    assert_eq!(textfilter::text2words(call_filter(json_str)), "bla cow\n");
}

#[test]
fn test_that_only_text_of_span_kept() {
    // this document contains a Span element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Span\", \"c\": [[\"\", [\"foo\"], []], [{\"t\": \"Str\", \"c\": \"Orc\"}]]}]}]]".into();
    assert!(call_filter(json_str).starts_with("Orc"));
}

#[test]
fn test_that_softbreak_is_correctly_serialized() {
    // this document contains a SoftBreak element, which should be ignored
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": \"a\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"SoftBreak\"},\
       {\"c\": \"b\", \"t\": \"Str\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "a b\n");
}
 
////////////////////////////////////////////////////////////////////////////////
// Block Elements

#[test]
fn test_that_plain_is_serialized_correctly() {
    // this document contains a Plain element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": \"plaintext\", \"t\": \"Str\"}],\
       \"t\": \"Plain\"}]]".into();
    assert_eq!(call_filter(json_str), "plaintext\n");
}


// this is actually redundant, but here it goes
#[test]
fn test_that_para_is_serialized_correctly() {
    // this document contains two paragraphs, a line break between both is expected
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": \"a\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"b\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"c\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"d\", \"t\": \"Str\"}],\
       \"t\": \"Para\"},\
       {\"c\": [{\"c\": \"e\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"f\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"g\", \"t\": \"Str\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "a b c d\ne f g\n");
}


#[test]
fn test_that_blockquote_is_serialized_correctly() {
    // this document contains a Blockquote element, text and author should be serialized
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"BlockQuote\", \"c\": [{\"t\": \"Para\", \"c\": [{\"t\": \"Str\", \"c\": \"never\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"eat\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"yellow\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"snow\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"SomeAuthor\"}]}]}]]".into();
    // don't test for equality, because there might be newline indicators at the end
    assert!(call_filter(json_str).starts_with("never eat yellow snow SomeAuthor"));
}


#[test]
fn test_that_orderedlist_is_serialized_correctly() {
    // this document contains a OrderedList element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [[1, {\"c\": [], \"t\": \"Decimal\"},\
       {\"c\": [], \"t\": \"Period\"}],\
       [[{\"c\": [{\"c\": \"shoe\", \"t\": \"Str\"}],\
       \"t\": \"Plain\"}],\
       [{\"c\": [{\"c\": \"jeans\", \"t\": \"Str\"}],\
       \"t\": \"Plain\"}],\
       [{\"c\": [{\"c\": \"cookie\", \"t\": \"Str\"}],\
       \"t\": \"Plain\"}]]], \"t\": \"OrderedList\"}]]".into();
    assert_eq!(call_filter(json_str), "shoe\njeans\ncookie\n");
}


#[test]
fn test_that_bulletlist_is_serialized_correctly() {
    // this document contains a BulletList element with each item forming a logical context
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"BulletList\", \"c\": [[{\"t\": \"Plain\", \"c\": [{\"t\": \"Str\", \"c\": \"first\"}]}],\
       [{\"t\": \"Plain\", \"c\": [{\"t\": \"Str\", \"c\": \"second\"}]}],\
       [{\"t\": \"Plain\", \"c\": [{\"t\": \"Str\", \"c\": \"third\"}]},\
       {\"t\": \"BulletList\", \"c\": [[{\"t\": \"Plain\", \"c\": [{\"t\": \"Str\", \"c\": \"third.one\"}]}]]}]]}]]".into();
    // each element (bullet point) is a context (in the word2vec sense), so newlines are expected
    assert_eq!(call_filter(json_str), "first\nsecond\nthird\nthird.one\n");
}


#[test]
fn test_that_definitionlist_is_serialized_correctly() {
    // this document contains a DefinitionList element; it's a definition list with two headwords
    // and two definitions
    let json_str: String = r#"[{"unMeta":{}},
      [{"t":"DefinitionList","c":[[[{"t":"Str","c":"triangle"}],
      [[{"t":"Plain","c":[{"t":"Str","c":"three"},
      {"t":"Space","c":[]},
      {"t":"Str","c":"edges"}]}]]],[[{"t":"Str","c":"square"}],
      [[{"t":"Plain","c":[{"t":"Str","c":"four"},
      {"t":"Space","c":[]},
      {"t":"Str","c":"edges"}]}]]]]}]]
      "#.into();
    assert_eq!(call_filter(json_str), "triangle three edges\nsquare four edges\n");
}


#[test]
fn test_that_header_is_serialized_correctly() {
    // this document contains a Header element (a heading)
    let json_str: String = r#"[{"unMeta": {}},
       [{"c": [1, ["h1", [], []], [{"c": "h1", "t": "Str"}]], "t": "Header"},
       {"c": [2, ["h2", [], []], [{"c": "h2", "t": "Str"}]], "t": "Header"}]]"#.into();
    assert_eq!(call_filter(json_str), "h1\nh2\n");
}


#[test]
fn test_that_only_text_in_div_kept() {
    // this document contains a Div element, only the content of the div should be kept
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [[\"\", [\"foobar\"], [[\"style\", \"ignored\"]]], [{\"c\": [{\"c\": \"content\", \"t\": \"Str\"}],\
       \"t\": \"Plain\"}]], \"t\": \"Div\"}]]".into();
    assert_eq!(call_filter(json_str), "content\n");
}
 

#[test]
fn test_that_codeblock_is_ignored_in_output() {
    // this document contains a CodeBlock element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": \"a\", \"t\": \"Str\"}],\
       \"t\": \"Para\"},\
       {\"c\": [[\"\", [], []], \"a\\nb\\nc\"], \"t\": \"CodeBlock\"},\
       {\"c\": [{\"c\": \"end\", \"t\": \"Str\"}],\
       \"t\": \"Para\"}]]".into();
    let result_copy = json_str.clone();
    assert!(call_filter(json_str).starts_with("a end"),
        format!("expected something starting with \"a end\", got \"{}\"", result_copy));
}


#[test]
fn test_that_table_is_ignored_in_output() {
    // this document contains a Table element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [[], [{\"c\": [], \"t\": \"AlignDefault\"},\
       {\"c\": [], \"t\": \"AlignDefault\"}],\
       [0, 0], [[{\"c\": [{\"c\": \"col1\", \"t\": \"Str\"}],\
       \"t\": \"Plain\"}],\
       [{\"c\": [{\"c\": \"col1\", \"t\": \"Str\"}],\
       \"t\": \"Plain\"}]], [[[{\"c\": [{\"c\": \"col1\", \"t\": \"Str\"}],\
       \"t\": \"Plain\"}],\
       [{\"c\": [{\"c\": \"col1\", \"t\": \"Str\"}],\
       \"t\": \"Plain\"}]]]], \"t\": \"Table\"}]]".into();
       // filter this to get empty string
    assert_eq!(textfilter::text2words(call_filter(json_str)), "");
}


#[test]
fn test_that_horizontalrule_is_ignored_in_output() {
    // this document contains a HorizontalRule element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Str\", \"c\": \"a\"}]},\
       {\"t\": \"HorizontalRule\", \"c\": []},\
       {\"t\": \"Para\", \"c\": [{\"t\": \"Str\", \"c\": \"b\"}]}]]".into();
    assert!(call_filter(json_str).starts_with("a b\n"));
}


#[test]
fn test_that_null_is_ignored_in_output() {
    // this document contains a Null element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Null\", \"c\": \"\"}]}]]".into();
    assert_eq!(call_filter(json_str), "");
}


#[test]
fn test_that_rawblock_is_ignored_in_output() {
    // this document contains a RawBlock element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [[\"\", [], []], \"<strong>ignore</strong>\"], \"t\": \"RawBlock\"}]]".into();
    assert_eq!(call_filter(json_str), "");
}
 
////////////////////////////////////////////////////////////////////////////////
// more behaviour tests

   
#[test]
#[should_panic]
fn test_that_json_documents_with_more_than_unmeta_and_content_are_incorrect() {
    // this document contains a jk element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Str\", \"c\": \"content\"}]}]\
       {\"invalid\":\"object\"]".into();
    call_filter(json_str);
}

