extern crate bzip2;
extern crate xml;

use bzip2::read::BzDecoder;
use std::fs::File;
use xml::reader::{EventReader, XmlEvent};

pub struct ArticleIterator {
    event_reader : EventReader<BzDecoder<File>>
}

impl ArticleIterator {
    fn new(filename: &str) -> ArticleIterator {
        let compressed = File::open(filename).unwrap();
        let er = EventReader::new(BzDecoder::new(compressed));
        ArticleIterator { event_reader: er }
    }
}

impl Iterator for ArticleIterator {
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
                        break;
                    }
                }
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

fn main() {
    for article in ArticleIterator::new("wikipedia.dump.bz2") {
        if !article.starts_with("#REDIRECT") {
            println!("article: {}\n-----------", article);
        }
    }
}

