use bzip2::read::BzDecoder;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use xml::reader::{EventReader, XmlEvent};

use super::super::input_source::Result;

macro_rules! get(
    ($e:expr) => (match $e {
        Some(e) => e,
        None => return None
    })
);




pub struct ArticleParser<B: Read> {
    event_reader: EventReader<B>
}

impl<B: Read> ArticleParser<B> {
    pub fn new(input_reader: B) -> ArticleParser<B> {
        let er = EventReader::new(input_reader);
        ArticleParser { event_reader: er }
    }
}

impl<B: Read> Iterator for ArticleParser<B> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Result<String>> {
        let mut is_text = false;
        let mut text = String::new();
        while let Ok(element) = self.event_reader.next() {
            match element {
                XmlEvent::StartElement { name, .. } => {
                    if name.local_name == "text" {
                        is_text = true;
                    }
                },
                XmlEvent::EndElement { name } => {
                    if name.local_name == "text" && is_text {
                        // check whether article text is only a redirect
                        if !text.starts_with("#REDIRECT") {
                            break;
                        }
                    }
                },
                XmlEvent::Characters(content) => {
                    if is_text {
                        text.push_str(&content);
                    }
                },
                XmlEvent::Whitespace(space) => {
                    if is_text {
                        text.push_str(&space);
                    }
            },
            _ => ()
            };
        };
        if !text.is_empty() {
            Some(Ok(text))
        } else {
            None
        }
    }
}

pub fn parser_from_file(filename: &Path) -> ArticleParser<BzDecoder<File>> {
    let compressed = File::open(filename.to_str().unwrap()).unwrap();
    ArticleParser::new(BzDecoder::new(compressed))
}

