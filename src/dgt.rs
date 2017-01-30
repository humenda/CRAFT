use pandoc;
use std::convert::From;
use std::fs;
use std::path::{Path, PathBuf};
use xml::reader::{EventReader, XmlEvent};
use zip::read::{ZipArchive, ZipFile};

use common;
use input_source::{GetIterator, TransformationError, Result, Unformatter};

// maximum buffer size of a String buffer parsed from XML
static MAX_BUFFER_SIZE: usize = 1048576; // 1M


struct DgtFiles {
    /// emit one path at a time from given directory
    zip_files: common::Files,
    /// current zip archive
    zip_archive: Option<ZipArchive<fs::File>>,
    /// entry index within current zip archive
    zip_entry: usize,
    /// length of zip file (so that zip_archive.unwrap().len() is not done each time)
    zip_entry_count: usize,
    /// language to filter for
    requested_language: String,
}

impl DgtFiles {
    // get new path from list of paths
    fn get_next_zip_archive(&mut self) -> Option<Result<ZipArchive<fs::File>>> {
        match self.zip_files.next() {
            Some(Ok(fpath)) => {
                let file = fs::File::open(fpath);
                if file.is_err() {
                    return None; // ToDo: properly return error
                }
                match ZipArchive::new(file.unwrap()) {
                    Ok(zip) => Some(Ok(zip)),
                    Err(e) => None, // ToDo: proper error handling
                }
            },
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    fn get_next_chunk(&mut self) -> Option<Result<String>> {
        // if no zip archive present or old zip file consumed, get new one
        if self.zip_archive.is_none() || self.zip_entry >= self.zip_entry_count {
            // open new, unread XML file
            match self.get_next_zip_archive() {
                None  => return None,
                Some(Err(e)) => return Some(Err(e)),
                Some(Ok(f)) => {
                    self.zip_entry_count = f.len();
                    self.zip_archive = Some(f);
                },
            };
        }

        // increment before returning for next iteration
        self.zip_entry += 1;
        let mut zip_archive = self.zip_archive.as_mut().unwrap();
        let mut zipfile = zip_archive.by_index(self.zip_entry - 1);
        if zipfile.is_err() {
            // ToDo: return error
            return None;
        }
        let zipfile = zipfile.unwrap();
        let file_length = zipfile.size();
        let mut evreader = EventReader::new(zipfile);

        // <tuv> nodes have "lang" attribute, which is compared against self.requested_language and
        // triggers requested_language_found set to true
        let mut requested_language_found = false;
        // buffer for uncompressed data
        let mut text = String::with_capacity(file_length as usize);

        let requested_language = self.requested_language.clone();
        while let Ok(element) = evreader.next() {
            match element {
                XmlEvent::StartElement { name, attributes, .. } =>
                    if let Some(_ign) = attributes.iter().find(|attr|
                            attr.name.local_name == "lang" &&
                            attr.value == requested_language) {
                        requested_language_found = true;
                },
                XmlEvent::EndElement { name } =>
                    if name.local_name == "seg" && requested_language_found {
                        requested_language_found = false;
                },
                XmlEvent::Characters(content) => {
                    if requested_language_found {
                        text.push_str(&content);
                        if text.len() >= MAX_BUFFER_SIZE {
                            break; // buffer "full"
                        }
                    }
                },
                XmlEvent::Whitespace(space) => {
                    if requested_language_found {
                        text.push_str(&space);
                    }
            },
            _ => ()
            };
        };
        match text.is_empty() {
            false => Some(Ok(text)),
            true => None
        }
    }
}

impl Iterator for DgtFiles {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_next_chunk()
    }
}


impl GetIterator for DgtFiles {
    fn iter(&self, input: &Path, language: Option<String>) -> Box<Iterator<Item=Result<String>>> {
        Box::new(DgtFiles {
            zip_files: common::Files::new(input, ".tmx".into()).unwrap(),
            zip_archive: None, zip_entry: 0, zip_entry_count: 0,
            requested_language: language.unwrap()
        })
    }
}

impl Unformatter for DgtFiles {
    fn is_preprocessing_required(&self) -> bool {
        false
    }

    fn get_input_format(&self) -> pandoc::InputFormat {
        pandoc::InputFormat::Markdown
    }

    fn preprocess(&self, input: &str) -> Result<String> {
        Ok(input.to_string())
    }
}

