use pandoc;
use std::fs::File;
use std::io::prelude::*;
use std::path;
use tempdir::TempDir;
use std::fs::OpenOptions;

use pandoc_filter;

// ToDo: remove
fn write_error(input: &str) {
    let mut file = OpenOptions::new().create(true).append(true).open("error.log").unwrap();
    file.write(format!("=-=-=-=-=-=-=-=-=-=-=-=-=-=\nError: {}\n",
                input).as_bytes()).unwrap();
}


pub struct PandocFilterer {
    tmpdir: TempDir
}


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
        let mut pandoc = pandoc::Pandoc::new();
        pandoc.set_output_format(pandoc::OutputFormat::Json);
        pandoc.add_input(&self.tmp_create_file(&input));
        pandoc.set_output("test.plain");
        pandoc.set_input_format(pandoc::InputFormat::MediaWiki);
        match pandoc.execute() {
            Ok(_) => (),
            Err(e) => {
            let text = format!("{:?}\nArticle:\n{}\n", e, input);
                write_error(&text);
            }
        };
        let plain_text = self.tmp_get_output("test.plain");
        pandoc_filter::stringify_text(plain_text)
    }
}

