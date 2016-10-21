use pandoc;
use std::fs::File;
use std::io::prelude::*;
use std::path;
use tempdir::TempDir;
use std::fs::OpenOptions;

// ToDo: remove
pub fn write_error(input: &str) {
    let mut file = OpenOptions::new().create(true).append(true).open("error.log").unwrap();
    file.write(format!("=-=-=-=-=-=-=-=-=-=-=-=-=-=\nError: {}\n",
                input).as_bytes()).unwrap();
}

#[derive(Default)]
pub struct MediawikiPreprocessor<'a> {
    in_table: bool,
    tag_start_found: bool,
    prevchar: char,
    original_data: &'a str,
    parsed_data: String,
    tmp_storage: String
}

// Todo: enforce mutability
impl<'a> MediawikiPreprocessor<'a> {
    pub fn new(input: &'a str) -> MediawikiPreprocessor<'a> {
        MediawikiPreprocessor { in_table: false, tag_start_found: false,
            original_data: input,
            ..Default::default()
        }
    }

    fn is_table_char(x: char) -> bool {
        x == '|' || x == '{' || x == '}'
    }

    // ToDo: proper error handling
    pub fn preprocess(&'a mut self) -> Result<&'a str, String> {
        for character in self.original_data.chars() {
            if self.in_table && !MediawikiPreprocessor::is_table_char(character) {
                self.prevchar = character;
                continue; // skip characters in tables
            }
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
        if self.tag_start_found {
            Err(format!("Unclosed tag, text after is: {}", self.tmp_storage))
        } else {
            Ok(&self.parsed_data)
        }
    }

    fn handle_table_character(&mut self, table_char: char) {
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
                println!("here: {}: {}", self.parsed_data, self.tmp_storage);
                self.tmp_storage.push(otherchar);
            }
        } else { // no tag start found, add vanilla
            self.parsed_data.push(otherchar);
        }
    }
}

// should be one per thread
pub struct PandocFilterer {
    tmpdir: TempDir
}

//fn clear_pandoc_ast(pandoc: &mut Pandoc) {
//    pandoc.add_filter(|json| pandoc_ast::filter(json, |mut pandoc| {
//        for block in &mut pandoc.1 {
//            use pandoc_ast::Block::*;
//            *block = match *block {
//                CodeBlock((_, ref kinds, _), _) if kinds.iter().next() == Some("graphviz") => {
//                    // do something to change a graphviz block into an image
//                }
//            }
//        }
//        pandoc
//    }));
//}

impl PandocFilterer {
    pub fn new() -> PandocFilterer {
        let tmpdir = TempDir::new("wikipedia2plain");
        PandocFilterer { tmpdir: tmpdir.unwrap() }
    }

    fn tmp_create_file(&self, input: &str) -> path::PathBuf {
        let fpath = self.tmpdir.path().join("test.mediawiki");
        let mut file = File::create(&fpath).unwrap();
        file.write_all(input.as_bytes()).unwrap();
        fpath
    }

    fn tmp_get_output(&self, fpath: &str) -> String {
        let mut file = File::open(fpath).unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        s
    }

    pub fn call_pandoc(&self, input: &str) -> String {
        //let input = preprocess(input);
        let mut pandoc = pandoc::Pandoc::new();
        pandoc.set_output_format(pandoc::OutputFormat::Plain);
        pandoc.add_input(&self.tmp_create_file(&input));
        pandoc.set_output("test.plain");
        pandoc.set_input_format(pandoc::InputFormat::MediaWiki);
        //clear_pandoc_ast(pandoc);
        match pandoc.execute() {
            Ok(_) => (),
            Err(e) => {
            let text = format!("{:?}\nArticle:\n{}\n", e, input);
                write_error(&text);
            }
        };
        self.tmp_get_output("test.plain")
    }
}

