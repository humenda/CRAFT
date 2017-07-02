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
/// - 24: error in configuration file

extern crate craft;
extern crate isolang;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate shellexpand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate textwrap;

use isolang::Language;
use log4rs::config::Config;
use log4rs::file::{Deserializers, RawConfig};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use craft::{common, textfilter};
use craft::modules::*;
use craft::input_source::{self, Entity, Unformatter};

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
/// Parse cmd options, return matches and input language
fn parse_cmd(program: &str, args: &[String])
        -> Result<(PathBuf, PathBuf), String> {
    let description: &'static str = "Crafted parses various input sources to \
        produce a word corpus, consisting only of words and numbers, with all \
        formatting, punctuation and special characters removed. \
        The result is written to the specified output file. If not done \
        carefully, it might overwrite processing results.";

    macro_rules! exists(
    ($thing:expr) => ({
        let p = PathBuf::from($thing.as_str());
        match p.exists() {
            true => p,
            false => return Err(format!("Given path {} doesn't exist.", $thing)),
        }
    }));
    match args.len() {
        1 => {
            if args[0] == "-h" || args[0] == "--help" {
                println!("Usage: {} <CONFIGURATION_YAML> [OUTPUT_FILE]\n\
                            {}", program, textwrap::fill(description, 80));
                ::std::process::exit(0);
            } else {
                println!("Not enough arguments.");
                println!("Usage: {} <CONFIGURATION_YAML> [OUTPUT_FILE]\n\
                            {}", program, textwrap::fill(description, 80));
                ::std::process::exit(1);
            }
        },
        2 => Ok((exists!(args[0].clone()), PathBuf::from(args[1].clone()))),
        _ => {
            println!("Wrong number of command line arguments.\n\
                Usage: {} <CONFIGURATION_YAML> <OUTPUT_FILE>\n\
                {}", program, textwrap::fill(description, 80));
            ::std::process::exit(1);
        },
    }
}

#[inline]
fn error_exit(msg: &str, exit_code: i32) {
    error!("{}", msg);
    ::std::process::exit(exit_code);
}

#[derive(Deserialize)]
struct LanguageCfg {
    wikipedia: Option<PathBuf>,
    gutenberg: Option<PathBuf>,
    dgt: Option<PathBuf>,
    europeana: Option<PathBuf>,
    codecivil: Option<PathBuf>,
    stopwords: Option<String>,
}

impl LanguageCfg {
    // find a better way to get string representation
    fn get_active_modules(&self) -> String {
        let mut active = String::new();
        {
            let mut add = |x| match active.len() {
                0 => active.push_str(x),
                _ => {
                    active.push_str(", ");
                    active.push_str(x);
                }
            };
            if self.wikipedia.is_some() {
                add("Wikipedia");
            }
            if self.gutenberg.is_some() {
                add("Gutenberg");
            }
            if self.dgt.is_some() {
                add("DGT (Translation Memories)");
            }
            if self.europeana.is_some() {
                add("Europeana");
            }
            if self.codecivil.is_some() {
                add("Code Civil");
            }
        }
        active
    }
}

#[derive(Deserialize)]
struct JointConfig {
    craft: HashMap<String, LanguageCfg>,
    log4rs: log4rs::file::RawConfig,
}

fn setup_config(log_conf: &PathBuf) -> HashMap<Language, LanguageCfg> {
    let cfg = ::serde_yaml::from_reader::<File, JointConfig>(
        File::open(log_conf).expect("Couldn't open log file for reading"))
        .unwrap(); // ToDo
    let ds = log4rs::file::Deserializers::new();
    let log_cfg = deserialize(&cfg.log4rs, &ds);
    log4rs::init_config(log_cfg).unwrap();

    // convert all keys to proper iso language objects
    let mut config = HashMap::new();
    for (key, value) in cfg.craft {
        match Language::from_639_3(&key) {
            Some(lang) => config.insert(lang, value),
            None => {
                error_exit(&format!("Invalid language in configuration: {}",
                        key), 24);
                unreachable!();
            }
        };
    }
    config
}

// 1-> 1 fork from log4rs, it wasn't public
fn deserialize(config: &RawConfig, deserializers: &Deserializers) -> Config {
    let (appenders, errors) = config.appenders_lossy(deserializers);
    for error in &errors {
        handle_error(error);
    }

    let (config, errors) = Config::builder()
        .appenders(appenders)
        .loggers(config.loggers())
        .build_lossy(config.root());
    for error in &errors {
        handle_error(error);
    }

    config
}

fn handle_error<E: ::std::error::Error + ?Sized>(e: &E) {
    let _ = writeln!(::std::io::stderr(), "log4rs: {}", e);
}
 

fn main() {
    let args: Vec<String> = env::args().collect();

    let (config_path, output_path) = trylog!(parse_cmd(&args[0], &args[1..]),
        "Errorneous command line", 1);

    let config = setup_config(&config_path);

    let mut result_file = match File::create(&output_path) {
        Err(e) => {
                error!("error while opening {} for writing: {}",
                       output_path.to_str().unwrap(), e);
                error_exit("please make sure that the output file is writable", 22);
                unreachable!();
            },
        Ok(f) => f
    };

    macro_rules! canonicalize(
        ($input_path:expr) => (
            $input_path.and_then(|p| match p.starts_with("~") {
                true => p.to_str().map(|p| ::shellexpand::tilde_with_context(p,
                        ::std::env::home_dir)).map(|p| PathBuf::from(p.into_owned())),
                false => Some(p),
            })
        )
    );

    for (lang, lconf) in config {
        info!("processing {}, active modules: {}", lang.to_name(),
            lconf.get_active_modules());
        if let Some(wp_path) = canonicalize!(lconf.wikipedia) {
            info!("extracting Wikipedia articles from {}",
                  wp_path.to_string_lossy());
            //extract_text(wikipedia::ArticleParser::new(
            //        trylog!(wikipedia::parser_from_file(&wp_path), "Could not open input file", 1)),
            //        Some(Box::new(wikipedia::Wikipedia)),
            //        &lconf.stopwords,
            //        &mut result_file);
            extract_text(trylog!(wikipedia::parser_from_file(&wp_path), "Could not open input file", 1),
                    Some(Box::new(wikipedia::Wikipedia)),
                    &lconf.stopwords,
                    &mut result_file);
        }
        if let Some(gb_path) = canonicalize!(lconf.gutenberg) {
            info!("Extracting Gutenberg books from {}",
                  gb_path.display());
            extract_text(common::read_files(gb_path.into(), "txt".into()),
                Some(Box::new(gutenberg::Gutenberg)),
                &lconf.stopwords,
                &mut result_file);
        }
        if let Some(europeana_path) = canonicalize!(lconf.europeana) {
            info!("Extracting news paper articles from {}",
                  europeana_path.to_string_lossy());
            let input_path = PathBuf::from(&europeana_path);
            extract_text(europeana::Articles::new(&input_path), None,
                &lconf.stopwords,
                &mut result_file);
        }
        if let Some(cc_path) = canonicalize!(lconf.codecivil) {
            info!("Extracting the code civil from {}",
                  cc_path.to_string_lossy());
            extract_text(common::read_files(cc_path.into(), "md".into()),
                Some(Box::new(codecivil::CodeCivil)),
                &lconf.stopwords,
                &mut result_file);
        }
        if let Some(dgt_path) = canonicalize!(lconf.dgt) {
            info!("extracting EU-DGT Translation Memories from {}",
                  dgt_path.to_string_lossy());
            extract_text(trylog!(dgt::DgtFiles::new(&dgt_path, lang.clone()), 
                "Unable to read from given directory", 2),
                None, &lconf.stopwords,
                &mut result_file);
        }
    }
}

/// Strip all formatting from text
///
/// This function utilises punctuation removing rules to get only plain text out of a document with
/// no formatting. If an unformatter is given, it will utilize Pandoc to extract the plain text
/// portions, before removing punctuation and stop words.
fn extract_text<Source: Iterator<Item=input_source::Result<Entity>>>(
        input_source: Source, unfmt: Option<Box<Unformatter>>,
        stopwords: &Option<String>,
        result_file: &mut File) {
    let mut entities_read = 0; // keep it external to for loop to retrieve later
    let mut errorneous_articles = 0;

    // an entity can be either an article, a book or similar, it's the smallest unit of processing
    for entity in input_source {
        entities_read += 1;
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
                    errorneous_articles += 1;
                    continue;
                }
            };
        }

        // strip white space, punctuation, non-character word-alike sequences, etc; keep only
        // single-space separated words (exception are line breaks for context conservation, see
        // appropriate module documentation)
        let stripped_words = match stopwords {
            &Some(ref words) => textfilter::text2words(entity.content, Some(words
                    .split(",").map(|x| x.trim().into()).collect::<HashSet<String>>())),
            &None => textfilter::text2words(entity.content, None),
        };
        if let Err(msg) = result_file.write_all(stripped_words.as_bytes()) {
            error!("could not write to output file: {}", msg);
            error_exit("Exiting", 23);
        }

        if (entities_read % 500) == 0 {
            info!("{} articles parsed, {} errorneous articles skipped.",
                entities_read, errorneous_articles);
        }
    }

    info!("{} articles read, {} were errorneous (and could not be included)",
        entities_read, errorneous_articles);
}

/// Remove formatting using pandoc
fn process_formatting<'a>(unfmt: &'a Unformatter, mut doc: Entity)
        -> input_source::Result<Entity> {
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

