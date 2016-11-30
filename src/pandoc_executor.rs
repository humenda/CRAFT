use pandoc;
use std::fs::File;
use std::io::prelude::*;
use std::path;
use tempdir::TempDir;
use std::fs::OpenOptions;

use input_source::*;

// ToDo: remove
fn write_error(input: &str) {
    let mut file = OpenOptions::new().create(true).append(true).open("error.log").unwrap();
    file.write(format!("=-=-=-=-=-=-=-=-=-=-=-=-=-=\nError: {}\n",
                input).as_bytes()).unwrap();
}


pub struct PandocFilterer {
    tmpdir: TempDir,
    format: pandoc::InputFormat,
}


impl PandocFilterer {
    pub fn new(input_format: pandoc::InputFormat) -> PandocFilterer {
        let tmpdir = TempDir::new("wikipedia2plain");
        PandocFilterer { tmpdir: tmpdir.unwrap(), format: input_format }
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

    pub fn call_pandoc(&self, input: &str) -> Result<String> {
        let mut pandoc = pandoc::Pandoc::new();
        pandoc.set_output_format(pandoc::OutputFormat::Json);
        pandoc.set_input_format(self.format.clone());
        pandoc.add_input(&self.tmp_create_file(&input));
        pandoc.set_output(pandoc::OutputKind::Pipe);
        match pandoc.execute() {
            Ok(pandoc::PandocOutput::ToBuffer(data)) => Ok(data),
            Ok(x) => panic!(format!("Expected converted data, got file name instead\nThis is a bug and needs to be fixed before continuing.")),
            Err(x) => Err(TransformationError::ErrorneousStructure(format!("{:?}\nArticle:\n{}\n",
                                                                           x, input), None))
        }
    }
}

