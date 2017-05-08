/// CRAFT — CorpoRA-based Freedict Translations
///
/// The CRAFT library and the crafted binary provide a corpus construction facility by parsing many
/// sources into one corpus which can be used for training word2vec. It is easy to include or
/// exclude certain data sources and also easy to add new ones.
///
/// The crafted binary has the following exit codes:
///
/// - 1: invalid cmd argument
/// - 2: invalid file name
/// - 22: - output not writable
/// - 23: error while writing to output

extern crate craft;
extern crate getopts;
extern crate isolang;
#[macro_use]
extern crate log;
extern crate log4rs;

use getopts::Options;
use isolang::Language;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use craft::*;
use craft::input_source::{self, Unformatter};

macro_rules! trylog(
    ($thing:expr, $msg:expr, $ret:expr) => (match $thing {
        Ok(x) => x,
        Err(ref e) => {
            error_exit(&format!("{}; {}", $msg, e), $ret);
            unreachable!();
        }
    };);
    ($thing:expr, $ret:expr) => (match $thing {
        Ok(e) => e,
        Err(ref e) => error_exit(format!("{}", e), $ret),
    });
);


// get program usage
fn get_usage(pname: &str, opts: Options) -> String {
    let description = "Crafted parses various input sources to produce a word corpus, which can then be \
        \nused by Word2vec. The output is written to a file called text8, which is over- \
        \nwritten on each launch.\n \
        The language for parsing has to be given as ISO 639-3 three-letter language code.\n";
    let usage = format!("Usage: {} [options, ...] LANGUAGE\n\n{}\n", pname, description);
    opts.usage(&usage)
}

/// Parse cmd options, return matches and input language
fn parse_cmd(program: &str, args: &[String])
        -> Result<(getopts::Matches, ::isolang::Language), String> {
    let mut opts = Options::new();
    opts.optopt("c", "codecivil", "activate code civil extractor, parsing \
                French laws from MarkDown files", "DIRECTORY");

    opts.optopt("e", "europeana", "activate europeana extractor for parsing news paper dumps",
                "DIRECTORY");
    opts.optopt("d", "eu-dgt", "activate EU-DGT (translation memory) \
                parser for the data of the EU DGT data in zip format",
                "DIRECTORY");
    opts.optopt("g", "gutenberg", "activate the Gutenberg corpus extractor \
                and read Gutenberg books from the specified path", "DIRECTORY");
    opts.optopt("l", "logging_conf", "set output file name for the logging \
                configuration (default log4rs.yaml)", "FILENAME");
    opts.optflag("h", "help", "print this help");
    opts.optopt("o", "output-file", "OUTPUT_FILE",
                r#"write to given output file, "text8" by default."#);

    opts.optopt("w", "wikipedia", "activate the Wikipedia corpus extractor \
                 and read Wikipedia articles from the specified bzipped XML \
                 article-only dump", "FILE");

    let matched = opts.parse(args);
    if matched.is_err() {
        return Err(format!("{}\n{}", matched.err().unwrap(),
                get_usage(program, opts)));
    }
    let matched = matched.unwrap();
    if matched.opt_present("h") {
        println!("{}", get_usage(program, opts));
        ::std::process::exit(0);
    }
    if !matched.opt_present("w") && !matched.opt_present("g") && !matched.opt_present("e") &&
        !matched.opt_present("c") && !matched.opt_present("d") &&
        !matched.opt_present("h") {
        return Err(format!("At least one output generator needs to be given.\n{}",
                           get_usage(program, opts)));
    }

    // get language
    if matched.free.len() < 2 {
        return Err(format!("The language to be parsed has to be given as a \
                    three-letter ISO 639-3 code.\n{}",
                           get_usage(program, opts)));
    } else {
        let lang = Language::from_639_3(&matched.free[1]).ok_or(format!(
                "Expected a valid ISO 639-3 code as language id, found {}", matched.free[1]))?;
        Ok((matched, lang))
    }
}

fn error_exit(msg: &str, exit_code: i32) {
    error!("{}", msg);
    ::std::process::exit(exit_code);
}

fn setup_logging(log_conf: &str) {
    if let Err(e) = log4rs::init_file(log_conf, Default::default()) {
        // ToDo: doesn't work, manual setup required
        warn!("Error while opening logging configuration {} for reading:\n      {}\
                \n    Logging to stdout instead", log_conf, e);
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let (opts, language) = trylog!(parse_cmd(program, &args[1..]),
        "Errorneous command line", 1);

    match opts.opt_present("l") {
        true => setup_logging(&opts.opt_str("l").unwrap()),
        false => setup_logging("log4rs.yaml"),
    }

    let output_name = opts.opt_str("o").unwrap_or("text8".into());
    let mut result_file = match File::create(&output_name) {
        Err(e) => {
                error!("error while opening {} for writing: {}", output_name, e);
                error_exit("please make sure that the output file is writable", 22);
                unreachable!();
            },
        Ok(f) => f
    };

    if let Some(wp_path) = opts.opt_str("w") {
        info!("extracting Wikipedia articles from {}", wp_path);
        let wp_path = PathBuf::from(wp_path);
        extract_text(wikipedia::ArticleParser::new(
            trylog!(File::open(wp_path), "Could not open input file", 1)),
                Some(Box::new(wikipedia::Wikipedia)),
                &mut result_file);
    }
    if let Some(gb_path) = opts.opt_str("g") {
        info!("Extracting Gutenberg books from {}", gb_path);
        extract_text(common::read_files(gb_path.into(), "txt".into()),
            Some(Box::new(gutenberg::Gutenberg)), &mut result_file);
    }
    if let Some(europeana_path) = opts.opt_str("e") {
        info!("Extracting news paper articles from {}", europeana_path);
        let input_path = PathBuf::from(&europeana_path);
        extract_text(europeana::Articles::new(&input_path), None,
            &mut result_file);
    }
    if let Some(cc_path) = opts.opt_str("c") {
        info!("Extracting the code civil from {}", cc_path);
        extract_text(common::read_files(cc_path.into(), "md".into()),
            Some(Box::new(codecivil::CodeCivil)), &mut result_file);
    }
    if let Some(dgt_path) = opts.opt_str("d") {
        info!("extracting EU-DGT notes from {}", dgt_path);
        let dgt_path = PathBuf::from(dgt_path);
        extract_text(trylog!(dgt::DgtFiles::new(&dgt_path, language.clone()), 
                "Unable to read from given directory", 2),
            None, &mut result_file);
    }
}

/// Strip all formatting from text
///
/// This function utilises punctuation removing rules to get only plain text out of a document with
/// no formatting. If an unformatter is given, it will utilize Pandoc to extract the plain text
/// portions, before removing punctuation and stop words.
fn extract_text<Source: Iterator<Item=input_source::Result<String>>>(
        input_source: Source, unfmt: Option<Box<Unformatter>>,
        result_file: &mut File) {
    let mut entities_read = 0; // keep it external to for loop to retrieve later
    let mut errorneous_articles = 0;

    // an entity can be either an article, a book or similar, it's the smallest unit of processing
    for entity in input_source {
        let mut entity = match entity {
            Ok(t) => t,
            Err(e) => {
                errorneous_articles += 1;
                debug!("unable to retrieve entity {} from input source; Error: {}",
                       entities_read, e);
                continue;
            }
        };

        if let Some(ref unfmt) = unfmt {
            entity = match process_formatting(&**unfmt, entity) {
                Ok(x) => x,
                Err(e) => {
                    error!("Error while preprocessing entity {}: {}",
                               entities_read, e);
                    return;
                }
            };
        }

        // strip white space, punctuation, non-character word-alike sequences, etc; keep only
        // single-space separated words (exception are line breaks for context conservation, see
        // appropriate module documentation)
        let stripped_words = textfilter::text2words(entity, None);
        if let Err(msg) = result_file.write_all(stripped_words.as_bytes()) {
            error!("could not write to output file: {}", msg);
            error_exit("Exiting", 23);
        }
        entities_read += 1;

        if (entities_read % 500) == 0 {
            info!("{} articles parsed, {} errorneous articles skipped.",
                entities_read, errorneous_articles);
        }
    }

    info!("{} articles read, {} were errorneous (and could not be included)",
        entities_read, errorneous_articles);
}

/// Remove formatting using pandoc
fn process_formatting<'a>(unfmt: &'a Unformatter, mut doc: String)
        -> input_source::Result<String> {
    // remove formatting which pandoc cannot handle (corner cases of incomplete
    // Pandoc readers)
    if unfmt.is_preprocessing_required() {
        doc = unfmt.preprocess(&doc)?
    }

    // retrieve a JSON representation of the document AST
    let json_ast = textfilter::call_pandoc(unfmt.get_input_format(), doc)?;

    // parse the text-only bits from the document
    textfilter::stringify_text(json_ast)
}

