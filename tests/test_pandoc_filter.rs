#[cfg(test)]
extern crate wikipedia2plain;

use wikipedia2plain::*;

fn call_filter(js_str: String) -> String {
    pandoc_filter::stringify_text(js_str)
}

////////////////////////////////////////////////////////////////////////////////
// test inline elements

#[test]
fn test_that_str_is_serialized_correctly() {
    // this document contains a Str element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Str\", \"c\": \"simplestr\"}]}]]".into(); 
    assert_eq!(call_filter(json_str), "simplestr");
}


#[test]
fn test_that_emph_is_serialized_correctly() {
    // this document contains a Emph element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Emph\", \"c\": [{\"t\": \"Str\", \"c\": \"emphasized\"}]}]}]]".into();
    assert_eq!(call_filter(json_str), "emphasized");
}


#[test]
fn test_that_strong_is_serialized_correctly() {
    // this document contains a Strong element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": [{\"c\": \"ok\", \"t\": \"Str\"}],\
       \"t\": \"Strong\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "ok");
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
    assert_eq!(call_filter(json_str), "let it be deleted.");
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
    assert_eq!(call_filter(json_str), "2 is pretty big");
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
    assert_eq!(call_filter(json_str), "drink enough H O");
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
    assert_eq!(call_filter(json_str), "UNO IEEE FOO");
}


#[test]
fn test_that_cite_is_serialized_correctly() {
    // this document contains a Cite element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": [[{\"citationPrefix\": [], \"citationId\": \"someauthor\", \"citationNoteNum\": 0, \"citationMode\": {\"c\": [], \"t\": \"NormalCitation\"},\
       \"citationHash\": 0, \"citationSuffix\": []}],\
       [{\"c\": \"[@someauthor]\", \"t\": \"Str\"}]], \"t\": \"Cite\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "");
}


#[test]
fn test_that_code_is_serialized_correctly() {
    // this document contains a Code element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": \"a\", \"t\": \"Str\"},\
       {\"c\": [[\"\", [], []], \"b\"], \"t\": \"Code\"},\
       {\"c\": \"c\", \"t\": \"Str\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "a c");
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
    assert_eq!(call_filter(json_str), "a b c d");
}


#[test]
fn test_that_linebreak_is_serialized_correctly() {
    // this document contains a LineBreak element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": \"here\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"LineBreak\"},\
       {\"c\": \"is\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"a\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"Space\"},\
       {\"c\": \"newline\", \"t\": \"Str\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "here is a newline");
}


#[test]
fn test_that_math_is_ignored() {
    // this document contains a Math element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Math\", \"c\": [{\"t\": \"InlineMath\", \"c\": []},\
       \"a\"]},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"and\"},\
       {\"t\": \"Space\", \"c\": []},\
       {\"t\": \"Math\", \"c\": [{\"t\": \"DisplayMath\", \"c\": []},\
       \"b\"]}]}]]".into();
    assert_eq!(call_filter(json_str), "and ");
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
    assert_eq!(call_filter(json_str), "this is ok");
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
    assert_eq!(call_filter(json_str), "an image of a beef steak");
}


// ToDo: does this element still exist?
//fn test_that_note_is_serialized_correctly() {
//}


#[test]
fn test_that_only_text_of_span_kept() {
    // this document contains a Span element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Span\", \"c\": [[\"\", [\"foo\"], []], [{\"t\": \"Str\", \"c\": \"Orc\"}]]}]}]]".into();
    assert_eq!(call_filter(json_str), "Orc");
}

#[test]
fn test_that_softbreak_is_correctly_serialized() {
    // this document contains a SoftBreak element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": \"a\", \"t\": \"Str\"},\
       {\"c\": [], \"t\": \"SoftBreak\"},\
       {\"c\": \"b\", \"t\": \"Str\"}],\
       \"t\": \"Para\"}]]".into();
    assert_eq!(call_filter(json_str), "a b");
}
 
////////////////////////////////////////////////////////////////////////////////
// Block Elements

#[test]
fn test_that_plain_is_serialized_correctly() {
    // this document contains a Plain element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [{\"c\": \"plaintext\", \"t\": \"Str\"}],\
       \"t\": \"Plain\"}]]".into();
    assert_eq!(call_filter(json_str), "plaintext");
}


// this is actually redundant, but here it goes
#[test]
fn test_that_para_is_serialized_correctly() {
    // this document contains a Para element
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
    assert_eq!(call_filter(json_str), "a b c d e f g");
}


#[test]
fn test_that_blockquote_is_serialized_correctly() {
    // this document contains a Blockquote element
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
    assert_eq!(call_filter(json_str), "never eat yellow snow SomeAuthor");
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
    assert_eq!(call_filter(json_str), "shoe jeans cookie");
}


#[test]
fn test_that_bulletlist_is_serialized_correctly() {
    // this document contains a BulletList element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"BulletList\", \"c\": [[{\"t\": \"Plain\", \"c\": [{\"t\": \"Str\", \"c\": \"first\"}]}],\
       [{\"t\": \"Plain\", \"c\": [{\"t\": \"Str\", \"c\": \"second\"}]}],\
       [{\"t\": \"Plain\", \"c\": [{\"t\": \"Str\", \"c\": \"third\"}]},\
       {\"t\": \"BulletList\", \"c\": [[{\"t\": \"Plain\", \"c\": [{\"t\": \"Str\", \"c\": \"third.one\"}]}]]}]]}]]".into();
    assert_eq!(call_filter(json_str), "first second third third.one");
}


#[test]
fn test_that_definitionlist_is_serialized_correctly() {
    // this document contains a DefinitionList element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"DefinitionList\", \"c\": [[[{\"t\": \"Str\", \"c\": \"headword\"}],\
       [[{\"t\": \"Plain\", \"c\": [{\"t\": \"Str\", \"c\": \"definition\"},\
       {\"t\": \"SoftBreak\", \"c\": []},\
       {\"t\": \"Str\", \"c\": \"another\"}]}],\
       [{\"t\": \"Plain\", \"c\": [{\"t\": \"Str\", \"c\": \"definition\"}]}]]]]}]]".into();
    assert_eq!(call_filter(json_str), "headword definition another definition");
}


#[test]
fn test_that_header_is_serialized_correctly() {
    // this document contains a Header element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [1, [\"h1\", [], []], [{\"c\": \"h1\", \"t\": \"Str\"}]], \"t\": \"Header\"},\
       {\"c\": [2, [\"h2\", [], []], [{\"c\": \"h2\", \"t\": \"Str\"}]], \"t\": \"Header\"}]]".into();
    assert_eq!(call_filter(json_str), "h1 h2");
}


#[test]
fn test_that_only_text_in_div_kept() {
    // this document contains a Div element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"c\": [[\"\", [\"foobar\"], [[\"style\", \"ignored\"]]], [{\"c\": [{\"c\": \"content\", \"t\": \"Str\"}],\
       \"t\": \"Plain\"}]], \"t\": \"Div\"}]]".into();
    assert_eq!(call_filter(json_str), "content");
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
    assert_eq!(call_filter(json_str), "a end");
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
    assert_eq!(call_filter(json_str), "");
}


#[test]
fn test_that_horizontalrule_is_ignored_in_output() {
    // this document contains a HorizontalRule element
    let json_str: String = "[{\"unMeta\": {}},\
       [{\"t\": \"Para\", \"c\": [{\"t\": \"Str\", \"c\": \"a\"}]},\
       {\"t\": \"HorizontalRule\", \"c\": []},\
       {\"t\": \"Para\", \"c\": [{\"t\": \"Str\", \"c\": \"b\"}]}]]".into();
    assert_eq!(call_filter(json_str), "a b");
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


