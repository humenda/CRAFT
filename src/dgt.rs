//! DGT-TM: European Translation Memories
//!
//! The source states:
//!
//! > DGT-TM is a translation memory (sentences and their manually produced translations) in 24
//! > languages. It contains segments from the Acquis Communautaire, the
//! > body of European legislation, comprising all the treaties, regulations and directives adopted
//! > by the European Union (EU). Since each new country joining the EU
//! > is required to accept the whole Acquis Communautaire, this body of legislation has been
//! > translated into 24 official languages. For the 23rd official EU
//! > language, Irish, the Acquis is not translated on a regular basis; which is why DGT-TM includes
//! > only a small amount of data in Irish.
//!
//! This module contains the required implementation to make this data usable. It parses all zip
//! files in a directory, looking for `*.tmx` files in those archives. The zip files can be
//! imported using the `eu-dgt.py` importer script in the importers directory.

use std::fs;
use std::io::{Read};
use std::iter;
use std::path::{Path};
use xml::reader::{EventReader, XmlEvent};
use zip::read::{ZipArchive};

use common;
use input_source::{GetIterator, Result, TransformationError};
use textfilter;

// maximum buffer size of a String buffer parsed from XML
static MAX_BUFFER_SIZE: usize = 1048576; // 1M


// An iterator over all (zip) files in a directory
//
// This iterator iterates over all *.zip files, opens them and within each zip archive, iterates
// over all *.tmx files. It decompreeesses each .tmx file and parses it with the XML event parser,
// storing the result in  a String and returning that.
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
    // get new path from list of paths (zip files)
    fn get_next_zip_archive(&mut self) -> Option<Result<()>> {
        match self.zip_files.next() {
            Some(fpath) => {
                let file = trysome!(fs::File::open(trysome!(fpath)));
                let zip = trysome!(ZipArchive::new(file)); // return ZipArchive
                self.zip_entry_count = zip.len();
                self.zip_archive = Some(zip);
                Some(Ok(()))
            },
            None => None,
        }
    }

    fn get_next_chunk(&mut self) -> Option<Result<String>> {
        // if no zip archive present or old zip file consumed, get new one
        if self.zip_archive.is_none() {
            // open new, unread XML file
            match get!(self.get_next_zip_archive()) {
                Err(e) => {
                    // increment file index even in case of error, otherwise endless loop
                    self.zip_entry += 1;
                    return Some(Err(e));
                },
                _ => (),
            }
        };

        // increment before returning for next iteration
        self.zip_entry += 1;
        let mut zip_archive = self.zip_archive.as_mut().unwrap();
        let mut zipfile = trysome!(zip_archive.by_index(self.zip_entry - 1));
        let file_length = zipfile.size() as usize;
        // load whole file nto RAM, decompress, decode from utf16 to utf8
        let mut raw_data = vec![0u8; file_length as usize];
        let mut total_bytes_read = 0;
        loop {
            let bytes_read = trysome!(zipfile.read(&mut raw_data[total_bytes_read..]));
            total_bytes_read += bytes_read;
            if total_bytes_read >= file_length {
                break;
            }
        }
        let raw_data = unsafe {
                ::std::slice::from_raw_parts_mut(raw_data.as_mut_ptr() as *mut u16,
                     raw_data.len() / 2) };
        let data = String::from_utf16(&raw_data[1..]).unwrap();
        let evreader = EventReader::new(::std::io::Cursor::new(data));

        // <tuv> nodes have "lang" attribute, which is compared against self.requested_language and
        // triggers requested_language_found set to true
        let mut requested_language_found = false;
        // buffer for parsed data
        let mut text = String::with_capacity(file_length / 4);
        let requested_language = self.requested_language.clone();
        for element in evreader {
            let element = trysome!(element);
            match element {
                XmlEvent::StartElement { name, attributes, .. } =>
                    // only <tuv lang="`self.requested_language`"> should match:
                    if name.local_name == "tuv" {
                        if let Some(_ign) = attributes.iter().find(|attr|
                            attr.name.local_name == "lang" &&
                            attr.value == requested_language) {
                            requested_language_found = true;
                    }
                },
                XmlEvent::EndElement { name } =>
                    if name.local_name == "seg" && requested_language_found {
                        requested_language_found = false;
                        text.push_str(&format!(" {} ",
                        textfilter::RETURN_ESCAPE_SEQUENCE));
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
            false => {
                if !text.ends_with("\n") {
                    text.push('\n'); // maintain word2vec "context" by adding newline
                }
                Some(Ok(text))
            },
            true => None
        }
    }
}

impl Iterator for DgtFiles {
    type Item = Result<String>;

    // get the plain text from the next parsed .tmx file, contained in one of    the zip archives
    fn next(&mut self) -> Option<Self::Item> {
        self.get_next_chunk()
    }
}



/// Input source for DGT-TM: European Translation Memories, [see  module docs](index.html)
pub struct Dgt;

impl GetIterator for Dgt {
    fn iter(&self, input: &Path, language: Option<String>) -> Box<Iterator<Item=Result<String>>> {

        Box::new(DgtFiles {
            zip_files: tryiter!(common::Files::new(input, "zip".into())),
            zip_archive: None, zip_entry: 0, zip_entry_count: 0,
            requested_language: tryiter!(language.ok_or(
                    TransformationError::InvalidInputArguments("No language \
                        supplied, which is required.".into())).into()),
        })
    }
}

