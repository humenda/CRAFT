use bzip2::read::BzDecoder;
use std::fs::File;
use xml::reader::{EventReader, XmlEvent};

pub struct ArticleParser {
    event_reader : EventReader<BzDecoder<File>>
}

impl ArticleParser {
    pub fn new(filename: &str) -> ArticleParser {
        let compressed = File::open(filename).unwrap();
        let er = EventReader::new(BzDecoder::new(compressed));
        ArticleParser { event_reader: er }
    }
}

impl Iterator for ArticleParser {
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

