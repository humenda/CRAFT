use bzip2::read::BzDecoder;
use std::io::Read;

use std::fs::File;
use std::path::Path;
use xml::reader::{EventReader, XmlEvent};

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
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut is_text = false;
        let mut text = String::new();
        loop {
            // ToDo: nice error handling, print error cause
            let element = self.event_reader.next().unwrap();
            let _ = match element {
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
            Some(text)
        } else {
            None
        }
    }
}

pub fn parser_from_file(filename: &Path) -> ArticleParser<BzDecoder<File>> {
    let compressed = File::open(filename.to_str().unwrap()).unwrap();
    ArticleParser::new(BzDecoder::new(compressed))
}

