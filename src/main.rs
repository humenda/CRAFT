/// CRAFT — CorpoRA-based Freedict Translations
///
/// The CRAFT library and the crafted binary provide a corpus construction facility by parsing many
/// sources into one corpus which can be used for training word2vec. It is easy to include or
/// exclude certain data sources and also easy to add new ones.
///
/// The crafted binary has the following exit codes:
///
/// - 1: invalid cmd argument
/// - 22: - output not writable
/// - 23: error while writing to output

extern crate craft;
extern crate getopts;
#[macro_use]
extern crate log;
extern crate log4rs;

use getopts::Options;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use craft::*;
use craft::input_source::{GetIterator, Unformatter};


// get program usage
fn get_usage(pname: &str, opts: Options) -> String {
    let description = "Crafted parses various input sources to produce a word corpus, which can then be \
        \nused by Word2vec. The output is written to a file called text8, which is over- \
        \nwritten on each launch.";
    let usage = format!("Usage: {} [options, ...]\n\n{}\n", pname, description);
    opts.usage(&usage)
}

/// Parse cmd options, return matches and input language
fn parse_cmd(program: &str, args: &[String])
        -> Result<(getopts::Matches, String), String> {
    let mut opts = Options::new();
    opts.optopt("c", "codecivil", "activate code civil extractor, parsing \
                French laws from MarkDown files", "DIRECTORY");

    opts.optopt("e", "europeana", "activate europeana extractor for parsing news paper dumps",
                "DIRECTORY");
    opts.optopt("d", "eu-dgta", "activate EU-DGT (translation memory) \
                parser for the data of the EU DGT data in zip format",
                "DIRECTORY");
    opts.optopt("g", "gutenberg", "activate the Gutenberg corpus extractor \
                and read Gutenberg books from the specified path", "DIRECTORY");
    opts.optopt("l", "logging_conf", "set output file name for the logging \
                configuration (default log4rs.yaml)", "FILENAME");
    opts.optflag("h", "help", "print this help");
    // ToDo: unused
    opts.optopt("o", "", "set output file name", "NAME");
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
    if matched.free.is_empty() {
        return Err(format!("The language to be parsed has to be given.\n{}",
                           get_usage(program, opts)));
    } else {
        let lang = matched.free[1].clone();
        Ok((matched, lang))
    }
}

fn error_exit(msg: &str, exit_code: i32) {
    println!("{}", msg);
    ::std::process::exit(exit_code);
}

fn setup_logging(log_conf: &str) {
    if let Err(e) = log4rs::init_file(log_conf, Default::default()) {
        // ToDo: doesn't work, manual setup required
        warn!("Error while opening logging configuration {} for reading:\n      {}\
                \n    Logging to stdout instead", log_conf, e);
    }
}

// Provide delegation to processing functionality
//
// Depending on the input source, the processing either consists of preprocessing, calling pandoc,
// converting the output of Pandoc to plain text and do post-processing or for the simpler cases,
// just do post-processing. The actual work is implemented in the corresponding input source, this
// enum just delegates the work accordingly.
enum Worker {
    Pandoc(String, Box<Unformatter>),
    PlainText(String),
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let opts = parse_cmd(program, &args[0..]);
    if opts.is_err() {
        error_exit(&format!("Error: {}", &opts.err().unwrap()), 1);
        return; // make compiler happy
    }
    let (opts, language) = opts.unwrap(); // safe now

    match opts.opt_present("l") {
        true => setup_logging(&opts.opt_str("l").unwrap()),
        false => setup_logging("log4rs.yaml"),
    }

    let output_name = opts.opt_str("o").unwrap_or("text8".into());
    let file_creation_result = File::create(&output_name);
    if file_creation_result.is_err() {
        error!("error while opening {} for writing: {}", output_name,
               file_creation_result.err().unwrap());
    
        error_exit("please make sure that the output file is writable", 22);
    } else {
        let mut result_file = file_creation_result.unwrap(); // safe now

        if let Some(wp_path) = opts.opt_str("w") {
            let input_path = Path::new(&wp_path);
            info!("extracting Wikipedia articles from {}", input_path.to_str().unwrap());
            let wikipedia = Box::new(wikipedia::Wikipedia);
            plain_text_with_pandoc(input_path, wikipedia, &mut result_file);
        }
        if let Some(gb_path) = opts.opt_str("g") {
            let input_path = Path::new(&gb_path);
            info!("Extracting Gutenberg books from {}", input_path.to_str().unwrap());
            let gutenberg = Box::new(gutenberg::Gutenberg);
            plain_text_with_pandoc(input_path, gutenberg, &mut result_file);
        }
        if let Some(europeana_path) = opts.opt_str("e") {
            let input_path = Path::new(&europeana_path);
            info!("Extracting news paper articles from {}", input_path.to_str().unwrap());
            let europeana = Box::new(europeana::Europeana);
            plain_text(input_path, europeana, &mut result_file, &language);
        }
        if let Some(cc_path) = opts.opt_str("c") {
            let input_path = Path::new(&cc_path);
            info!("Extracting the code civil from {}", input_path.to_str().unwrap());
            let codecivil = Box::new(codecivil::CodeCivil);
            plain_text_with_pandoc(input_path, codecivil, &mut result_file);
        }
        if let Some(dgt_path) = opts.opt_str("d") {
            info!("extracting EU-DGT notes from {}", dgt_path);
            let input_path = Path::new(&dgt_path);
            let dgt = Box::new(dgt::Dgt);
            plain_text(input_path, dgt, &mut result_file, &language);
        }
    }
}


// This macro matches on a result; if `Ok()`, take it, else log the error output and increment the
// given error counter.
macro_rules! use_or_skip(
    ($matchon:expr, $inc_on_error:tt, $warn_format_str:tt,
           $($warn_args:tt),*) => (match $matchon {
        Ok(t) => t,
        Err(e) => {
            // print custom error message and include error in the logs, too
            $inc_on_error += 1;
            debug!(concat!($warn_format_str, "\n    Error: {}"),
                $($warn_args),*, e);
            continue;
        }
    })
);

/// Strip all formatting from a text
///
/// This function utilises pandoc and punctuation removing rules to get only plain text out of a
/// formatted document.
fn plain_text_with_pandoc<Source: GetIterator + Unformatter>(input: &Path,
                  input_source: Box<Source>, result_file: &mut File) {
    let mut entities_read = 0; // keep it external to for loop to retrieve later
    let mut errorneous_articles = 0;

    // an entity can be either an article, a book or similar, it's the smallest unit of processing;
    // the second parameter to iter is the language to be parsed (this is not supported for this
    // kind of input sources, yet)
    for entity in input_source.iter(input, None) {
        let mut entity = use_or_skip!(entity, errorneous_articles,
            "unable to retrieve entity {} from input source", entities_read);
        debug!("Starting to process entity {} with length {}", entities_read, entity.len());

        // preprocessing might remove formatting which Pandoc cannot handle
        if input_source.is_preprocessing_required() {
            entity = use_or_skip!(input_source.preprocess(&entity),
                errorneous_articles, "unable to preprocess entity {}",
                entities_read);
        }

        // retrieve a JSON representation of the document AST
        let json_ast = use_or_skip!(
                textfilter::call_pandoc(input_source.get_input_format(), entity),
                errorneous_articles,
                "entity {} couldn't be parsed by pandoc", entities_read);

        // parse the text-only bits from the document
        entity = use_or_skip!(textfilter::stringify_text(json_ast), errorneous_articles,
            "unable to extract plain text from Pandoc document AST for entity {}", entities_read);

        // strip white space, punctuation, non-character word-alike sequences, etc; keep only
        // single-space separated words (exception are line breaks for context conservation, see
        // appropriate module documentation)
        let stripped_words = textfilter::text2words(entity);
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

/// Strip all formatting from a text
///
/// This function utilises punctuation removing rules to get only plain text out of a document with
/// no formatting.
fn plain_text<Source: GetIterator>(input: &Path, input_source: Box<Source>,
           result_file: &mut File, language: &String) {
    let mut entities_read = 0; // keep it external to for loop to retrieve later
    let mut errorneous_articles = 0;

    // an entity can be either an article, a book or similar, it's the smallest unit of processing
    for entity in input_source.iter(input, Some(language.clone())) {
        let entity = use_or_skip!(entity, errorneous_articles,
            "unable to retrieve entity {} from input source", entities_read);

        // strip white space, punctuation, non-character word-alike sequences, etc; keep only
        // single-space separated words (exception are line breaks for context conservation, see
        // appropriate module documentation)
        let stripped_words = textfilter::text2words(entity);
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

