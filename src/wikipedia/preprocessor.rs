use super::super::input_source::{TransformationError, Result};

#[derive(Default)]
pub struct MediawikiPreprocessor<'a> {
    /// original mediawiki input
    original_data: &'a str,
    /// indicate whether a table has been encountered; use to ignore everything inside
    ignore_content: bool,
    /// indicatation of an opened HTML tag
    tag_start_found: bool,
    /// store previous character
    prevchar: char,
    /// parsed output data with some things modified / removed
    parsed_data: String,
    /// temporary storage, e.g. when parsing a tag
    tmp_storage: String
}

impl<'a> MediawikiPreprocessor<'a> {
    /// obtain a new instance of a MediawikiPreprocessor with a given mediawiki input string
    pub fn new(input: &'a str) -> MediawikiPreprocessor<'a> {
        MediawikiPreprocessor { ignore_content: false, tag_start_found: false,
            original_data: input,
            ..Default::default()
        }
    }

    // test whether character is one of the table identifiers within mediawiki
    fn is_table_char(x: char) -> bool {
        x == '|' || x == '{' || x == '}'
    }

    // ToDo: proper error handling
    pub fn preprocess(&'a mut self) -> Result<String> {
        for character in self.original_data.chars() {
            // ignore characters within a table

            // try to identify tables and HTML tags by looking at each character (and the previous
            // one)
            match character {
                tb if MediawikiPreprocessor::is_table_char(tb) => self.handle_table_character(tb),
                lt if lt == '<' => self.tag_start_found = true,
                // NOTE: also see handle_other_character() for tag handling(!)
                gt if gt == '>' => self.handle_sgml_tag(),
                otherchar => self.handle_other_character(otherchar),
            };
            self.prevchar = character;
        }

        // if tag_start_found still set, unclosed HTML tag
        if self.tag_start_found {
            Err(TransformationError::ErrorneousStructure(
                    format!("text after opening <: {}",
                    self.tmp_storage), None))
        } else {
            // ToDo: Hendrik, wie ohne clone?
            Ok(self.parsed_data.clone())
        }
    }

    fn found_tag(&self, tag: &str) -> bool {
        self.tmp_storage.starts_with(tag) || self.tmp_storage.starts_with(&tag.to_lowercase())
    }

    /// get rid of the `<ref/>` tags (with their content) and the blockquote tags (keeping their
    /// content)
    fn handle_sgml_tag(&mut self) {
        if !self.tag_start_found { // only trigger if a < has been found before
            if !self.ignore_content {
                self.parsed_data.push('>');
            }
            return;
        }

        self.tag_start_found = false;
        if self.found_tag("ref") {
            self.ignore_content = true;
        } else if self.found_tag("/ref") { // read content from now on
            self.ignore_content = false;
        // if it's not a block quote
        } else if self.found_tag("/blockquote") || self.found_tag("blockquote") {
            // discard the blockquote tag
        } else { // add tag vanilla to output
            self.parsed_data.push_str(&format!("<{}>",
                                               self.tmp_storage));
        }
        self.tmp_storage.clear();
    }

    fn handle_table_character(&mut self, table_char: char) {
        // This function figures out whether the encountered "table character" was the second in a
        // row, and therefore opens a table. In more straight words, if | is followed by { for { by
        // |, this opens (or other way around, closes) a table.
        // table is on newline, if character _before_ previous character is \n
        let table_on_newline = |x: &String| match x.chars().rev().skip(1).next() {
            Some(x) if x == '\n' => true,
            None => true, // if nothing, that's a newline too
            _ => false
        };

        // tables may start on |{ or {| and end on }| or |}; identifying a table is
        // hence a two-fold process: detect first character and realize that
        // character before was _not_ a table character and then identify second,
        // identify the previous as table character too and identify subsequent
        // stuff as table
        if MediawikiPreprocessor::is_table_char(self.prevchar) && 
                self.prevchar != table_char {
            // if not identified same character twice and is at start of line
            if self.ignore_content {
                self.ignore_content = false;
            } else {
                if table_on_newline(&self.parsed_data) {
                    self.ignore_content = true;
                    // in previous step, function might have added character to
                    // parsed_data, but that's a table char, remove it
                    if self.parsed_data.ends_with(self.prevchar) {
                        let _ = self.parsed_data.pop();
                    }
                } else { // random |{ or {| on a line, no table, add vanilla
                    self.parsed_data.push(table_char);
                }
            }
        } else { // self.prevchar is not a table char
            // only if we're not in a table (and ignore its content), add it ad a character
            if !self.ignore_content {
                self.parsed_data.push(table_char);
            }
        }
    }

    // normal characters:
    // 1. within table: discarded
    // 2. within html tag: check whether first recognized character is b or r, otherwise tag
    //    parsing abborted (only interested in <blockquote...>
    // 3. add it to parsed text
    fn handle_other_character(&mut self, otherchar: char) {
        // ignore all content, except for tags (might mark end-of-ignore section
        if self.ignore_content && !self.tag_start_found {
            return;
        }

        // characters in tables are discarded, so check whether in table
        // parsing a HTML tag? `<..>`?
        if self.tag_start_found {
            // only <blockquote>, </blockquote>, <ref> und </ref> matters, ignore all other
            // tags or even straying < signs; this is a slight performance improvement, no string
            // needs to grow holding the tag
            if self.prevchar == '<' && otherchar != 'b' && otherchar != 'r' && otherchar != '/' {
                self.parsed_data.push('<');
                self.parsed_data.push(otherchar);
                self.tag_start_found = false;
            } else { // is a <b... tag, save text to examine when > found
                self.tmp_storage.push(otherchar);
            }
        } else { // no tag start found, add vanilla
            self.parsed_data.push(otherchar);
        }
    }
}


