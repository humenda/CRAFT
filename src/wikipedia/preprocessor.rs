#[derive(Default)]
pub struct MediawikiPreprocessor<'a> {
    /// indicate whether a table has been encountered; use to ignore everything inside
    in_table: bool,
    /// indicatation of an opened HTML tag
    tag_start_found: bool,
    /// store previous character
    prevchar: char,
    /// original mediawiki input
    original_data: &'a str,
    /// parsed output data with some things modified / removed
    parsed_data: String,
    /// temporary storage, e.g. when parsing a tag
    tmp_storage: String
}

impl<'a> MediawikiPreprocessor<'a> {
    /// obtain a new instance of a MediawikiPreprocessor with a given mediawiki input string
    pub fn new(input: &'a str) -> MediawikiPreprocessor<'a> {
        MediawikiPreprocessor { in_table: false, tag_start_found: false,
            original_data: input,
            ..Default::default()
        }
    }

    // test whether character is one of the table identifiers within mediawiki
    fn is_table_char(x: char) -> bool {
        x == '|' || x == '{' || x == '}'
    }

    // ToDo: proper error handling
    pub fn preprocess(&'a mut self) -> Result<String, String> {
        for character in self.original_data.chars() {
            // ignore characters within a table
            if self.in_table && !MediawikiPreprocessor::is_table_char(character) {
                self.prevchar = character;
                continue; // skip characters in tables
            }

            // try to identify tables and HTML tags by looking at each character (and the previous
            // one)
            match character {
                tb if MediawikiPreprocessor::is_table_char(tb) => self.handle_table_character(tb),
                lt if lt == '<' => self.tag_start_found = true,
                gt if gt == '>' => {
                    if self.tag_start_found { // only trigger if a < has been found before
                        // if it's not a block quote, add it as normal tag
                        if !(self.tmp_storage.starts_with("/blockquote") ||
                                self.tmp_storage.starts_with("blockquote")) {
                            self.parsed_data.push_str(&format!("<{}>",
                                        self.tmp_storage));
                        } // else: // is block quote, discard tags
                        self.tag_start_found = false;
                        self.tmp_storage.clear(); // always discard temporary parsing result
                    } else { // straying >, add it
                        self.parsed_data.push('>');
                    }
                },
                otherchar => self.handle_other_character(otherchar),
            };
            self.prevchar = character;
        }

        // if tag_start_found still set, unclosed HTML tag
        if self.tag_start_found {
            Err(format!("Unclosed tag, text after is: {}", self.tmp_storage))
        } else {
            // ToDo: Hendrik, wie ohne clone?
            Ok(self.parsed_data.clone())
        }
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
            if self.in_table {
                self.in_table = false;
            } else {
                if table_on_newline(&self.parsed_data) {
                    self.in_table = true;
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
            // only if we're not in a table, add it ad a character
            if !self.in_table {
                self.parsed_data.push(table_char);
            }
        }
    }

            // normal characters:
            // 1. within table: discarded
            // 2. within html tag: check whether first recognized character is b, otherwise tag
            //    parsing abborted (only interested in <blockquote...>
            // 3. add it to parsed text
    fn handle_other_character(&mut self, otherchar: char) {
        // characters in tables are discarded, so check whether in table
        if self.in_table { // characters in tables are discarded
            return;
        }
        // parsing a HTML tag? `<..>`?
        if self.tag_start_found {
            // only <blockquote> and </blockquote> matters, ignore all other
            // tags or even straying < signs
            if self.prevchar == '<' && otherchar != 'b' && otherchar != '/' {
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


