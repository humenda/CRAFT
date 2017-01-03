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
use craft::gutenberg::Gutenberg;
use craft::input_source::InputSource;
use craft::wikipedia::Wikipedia;


fn get_usage(pname: &str, opts: Options) -> String {
    let description = "Crafted parses various input sources to produce a word corpus, which can then be \
        \nused by Word2vec. The output is written to a file called text8, which is over- \
        \nwritten on each launch.";
    let usage = format!("Usage: {} [options, ...]\n\n{}\n", pname, description);
    opts.usage(&usage)
}

fn parse_cmd(program: &str, args: &[String]) -> Result<getopts::Matches, String> {
    let mut opts = Options::new();
    opts.optopt("e", "europeana", "activate europeana extractor for parsing news paper dumps",
                "DIRECTORY");
    opts.optopt("g", "gutenberg", "activate the Gutenberg corpus extractor \
                and read Gutenberg books from the specified path", "DIRECTORY");
    opts.optflag("h", "help", "print this help");
    // ToDo: unused
    opts.optopt("o", "", "set output file name", "NAME");
    opts.optopt("w", "wikipedia", "activate the Wikipedia corpus extractor \
                 and read Wikipedia articles from the specified bzipped XML \
                 article-only dump", "FILE");

    let matched = opts.parse(args);
    if matched.is_err() {
        return matched.map_err(|f| format!("{}\n{}", f.to_string(),
                get_usage(program, opts)));
    }
    let matched = matched.unwrap();
    if matched.opt_present("h") {
        println!("{}", get_usage(program, opts));
        ::std::process::exit(0);
    }
    if !matched.opt_present("w") && !matched.opt_present("g") && !matched.opt_present("e") &&
        !matched.opt_present("h") {
        return Err(format!("At least one output generator needs to be given.\n{}",
                           get_usage(program, opts)));
    }

    Ok(matched)
}

fn error_exit(msg: &str, exit_code: i32) {
    println!("{}", msg);
    ::std::process::exit(exit_code);
}

fn setup_logging() {
    log4rs::init_file("log4rs.yaml", Default::default()).expect("could not open log file for writing!");
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let opts = parse_cmd(program, &args[0..]);
    if opts.is_err() {
        error_exit(&format!("Error: {}", &opts.err().unwrap()), 1);
        return; // make compiler happy
    }
    let opts = opts.unwrap(); // safe now

    setup_logging();

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
            let wikipedia = Box::new(Wikipedia); // ToDo
            make_corpus(input_path, wikipedia, &mut result_file);
        }
        if let Some(gb_path) = opts.opt_str("g") {
            let input_path = Path::new(&gb_path);
            info!("Extracting Gutenberg books from {}", input_path.to_str().unwrap());
            let gutenberg = Box::new(Gutenberg);
            make_corpus(input_path, gutenberg, &mut result_file);
        }
        if let Some(europeana_path) = opts.opt_str("e") {
            let input_path = Path::new(&europeana_path);
            info!("Extracting news paper articles from {}", input_path.to_str().unwrap());
            let europeana = Box::new(europeana::Europeana);
            make_corpus(input_path, europeana, &mut result_file);
        }
    }
}

fn make_corpus(input: &Path, input_source: Box<InputSource>, result_file: &mut File) {
                let mut articles_read = 0; // keep it external to for loop to retrieve later
    let mut errorneous_articles = 0;
    let pandoc = pandoc_executor::PandocFilterer::new(input_source.get_input_format());

    for article in input_source.get_input(input) {
        let mut article = match article {
            Ok(a) => a,
            Err(e) => {
                errorneous_articles += 1;
                debug!("got errorneous article: {:?}", e);
                continue;
            }
        };
        if input_source.is_preprocessing_required() {
            article = match input_source.preprocess(&article) {
                // ToDo: put that into some kind of log file
                Err(_) => {
                    errorneous_articles += 1;
                    continue;
                },
                Ok(x) => x,
            };
        }

        let json_ast = match pandoc.call_pandoc(&article) {
            Ok(t) => t,
            Err(e) => {
                errorneous_articles += 1;
                warn!("entity {} culdn't be parsed with pandoc", articles_read);
                debug!("error: {:?}", e);
                continue;
            }
        };
        article = text2plain::stringify_text(json_ast);

        let stripped_words = format!("{}\n", text2plain::text2words(article));
        if let Err(msg) = result_file.write_all(stripped_words.as_bytes()) {
            error!("could not write to output file: {}", msg);
            error_exit("Exiting", 23);
        }
        articles_read += 1;

        
        if (articles_read % 500) == 0 {
            info!("{} articles parsed, {} errorneous articles skipped.",
                articles_read, errorneous_articles);
        }
    }

    info!("{} articles read, {} were errorneous (and could not be included)",
        articles_read, errorneous_articles);
}


