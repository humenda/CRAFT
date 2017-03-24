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

use isolang::Language;
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
    /// ISO 639-1 language code to filter for
    requested_language: String,
    /// helper to figure out whether zip_archive is None because end reached or first iteration
    iteration_started: bool,
}

impl DgtFiles {
    // this method opens the next zip archive and saves it into self.zip_archive
    fn get_next_zip_archive(&mut self) -> Option<Result<()>> {
        let fpath = get!(self.zip_files.next());
        let file = trysome!(fs::File::open(trysome!(fpath)));
        let zip = trysome!(ZipArchive::new(file));
        self.zip_entry = 0;
        self.zip_entry_count = zip.len(); // number of files in zip
        self.zip_archive = Some(zip);
        Some(Ok(()))
    }

    // Read the contents of the given entry to RAM. The callee must make sure that 
    // `self.zip_archive != None` and `self.zip_entry < self.zip_entry_count`
    fn read_zip_entry_to_ram(&mut self, index: usize) -> Result<String> {
        let mut zip_archive = self.zip_archive.as_mut().unwrap(); // safe here
        let mut zipped_file = try!(zip_archive.by_index(index));
        let file_length = zipped_file.size() as usize;
        // load whole file to RAM, decompress, decode from utf16 to utf8
        let mut raw_data = vec![0u8; file_length as usize];
        let mut total_bytes_read = 0;
        loop {
            let bytes_read = try!(zipped_file.read(&mut raw_data[total_bytes_read..]));
            total_bytes_read += bytes_read;
            if total_bytes_read >= file_length {
                break;
            }
        }
        // UTF-16 can only be read from u16, but data can be only read in byte granularity, so
        // reinterpreting memory is required (platform portability?)
        if (raw_data.len() % 2) == 1 {
            return Err(TransformationError::EncodingError("DGT input data \
                    couldn't be decoded as UtF-16, uneven byte count.".into()));
        }
        let raw_data = unsafe {
                ::std::slice::from_raw_parts_mut(raw_data.as_mut_ptr() as *mut u16,
                     raw_data.len() / 2) };
        match raw_data[0] == 0xfeff {
            true => String::from_utf16(&raw_data[1..]).map_err(|_|
                TransformationError::EncodingError("Invalid UTF16-encoded file".into())),
            false => String::from_utf16(&raw_data).map_err(|_|
                TransformationError::EncodingError("Invalid UTF16-encoded file".into())),
        }
    }

    // parse the XML file and write the output to the second input parameter
    fn parse_xml(&self, xml: String, output: &mut String) -> Result<()> {
        let evreader = EventReader::new(::std::io::Cursor::new(xml));

        // <tuv> nodes have "lang" attribute, which is compared against self.requested_language and
        // triggers requested_language_found set to true
        let mut requested_language_found = false;
        let requested_language = self.requested_language.clone();
        for element in evreader {
            let element = element?;
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
                        output.push_str(&format!(" {} ",
                                                         textfilter::RETURN_ESCAPE_SEQUENCE));
                },
                XmlEvent::Characters(content) => if requested_language_found {
                        output.push_str(&content);
                        if output.len() >= MAX_BUFFER_SIZE {
                            break; // buffer "full"
                        }
                },
                XmlEvent::Whitespace(space) => if requested_language_found {
                        output.push_str(&space);
                },
                _ => ()
            };
        }
        Ok(())
    }

    fn get_next_chunk(&mut self) -> Option<Result<String>> {
        // loop until zip archive or zip entry with request data is found
        let mut extracted_text = String::new();
        loop {
            if !self.iteration_started {
                self.iteration_started = true;
            }

            // if no zip archive present or old zip file consumed, get new one
            if self.zip_archive.is_none() {
                if let Err(err) = get!(self.get_next_zip_archive()) {
                    self.zip_entry += 1; // basically ignores this entry
                    return Some(Err(err));
                }
            } else if self.zip_entry >= self.zip_entry_count {
                self.zip_archive = None;
                continue; // done with this zip file, go and fetch a new one
            }

            // increment before returning for next iteration
            self.zip_entry += 1;
            let current_index = self.zip_entry - 1;
            let data = trysome!(self.read_zip_entry_to_ram(current_index));
            extracted_text.reserve(data.len() / 4);
            trysome!(self.parse_xml(data, &mut extracted_text));
            if !extracted_text.is_empty() {
                if !extracted_text.ends_with("\n") {
                    extracted_text.push('\n'); // maintain word2vec "context" by adding newline
                }
                return Some(Ok(extracted_text))
            } // otherwise: loooooop
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
    fn iter(&self, input: &Path, language: Option<Language>)
            -> Box<Iterator<Item=Result<String>>> {
        // get language and convert into 639-1
        let lang = tryiter!(language.ok_or(TransformationError::InvalidInputArguments(
                "No language supplied, which is required.".into())).into());
        let lang = tryiter!(lang.to_639_1().ok_or(
                    TransformationError::InvalidInputArguments(format!(
                        "Requested language {} doesn't have a ISO 639-1 two-\
                        letter language code", lang.to_639_3()))));

        Box::new(DgtFiles {
            zip_files: tryiter!(common::Files::new(input, "zip".into())),
            zip_archive: None, zip_entry: 0, zip_entry_count: 0,
            requested_language: lang.into(),
            iteration_started: false,
        })
    }
}

