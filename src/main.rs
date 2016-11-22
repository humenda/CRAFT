/// ToDo: document exit codes
/// 1 -> invalid cmd argument
/// 2 -> input file / directory not found

extern crate getopts;
extern crate wikipedia2plain;

use getopts::Options;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use wikipedia2plain::*;
use wikipedia2plain::gutenberg::Gutenberg;
use wikipedia2plain::input_source::InputSource;
use wikipedia2plain::wikipedia::Wikipedia;

fn get_usage(pname: &str, opts: Options) -> String {
    let usage = format!("Usage: {} [options, ...]\n\n", pname);
            opts.usage(&usage)
}

fn parse_cmd(program: &str, args: &[String]) -> Result<getopts::Matches, String> {
    let mut opts = Options::new();
    opts.optopt("o", "", "set output file name", "NAME");
    opts.optopt("g", "gutenberg", "activate the Gutenberg corpus extractor \
                 and read Gutenberg books from the specified path", "DIRECTORY");
    opts.optflag("h", "help", "print this help");
    opts.optopt("w", "wikipedia", "activate the Wikipedia corpus extractor \
                 and read Wikipedia articles from the specified bzipped XML \
                 article-only dump", "FILE");

    let matched = opts.parse(args);
    if matched.is_err() {
        return matched.map_err(|f| format!("{}\n{}", f.to_string(),
                get_usage(program, opts)));
    }
    let matched = matched.unwrap();
    if !matched.opt_present("w") && !matched.opt_present("g") {
        return Err(format!("At least one output generator needs to be given.\n{}",
                           get_usage(program, opts)));
    }

    Ok(matched)
}

fn error_exit(msg: String, exit_code: i32) {
    println!("{}", msg);
    ::std::process::exit(exit_code);
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let opts = parse_cmd(program, &args[0..]);
    if opts.is_err() {
        error_exit(format!("Error: {}", &opts.err().unwrap()), 1);
        return; // make compiler happy
    }
    let opts = opts.unwrap(); // safe now

    let mut result_file = File::create("text8").unwrap();

    let input_path = opts.opt_str("w").expect("Required input file not found.");
    let input_path = Path::new(&input_path);
    let w = Box::new(Wikipedia); // ToDo
    make_corpus(input_path, w, &mut result_file);
}

fn make_corpus(input: &Path, input_source: Box<InputSource>, result_file: &mut File) {
    let mut articles_read = 0;
    let mut errorneous_articles = 0;
    let pandoc = pandoc_executor::PandocFilterer::new();

    for article in input_source.get_input(input) {
        let article = match article {
            Ok(a) => a,
            Err(_) => {
                errorneous_articles += 1;
                continue;
            }
        };
        let article = match input_source.preprocess(&article) {
            // ToDo: put that into some kind of log file
            Err(_) => {
                errorneous_articles += 1;
                continue;
            },
         Ok(x) => x,
        };
        let json_ast = pandoc.call_pandoc(&article);
        let article = text2plain::stringify_text(json_ast);

        let stripped_words = text2plain::text2words(article);
        result_file.write_all(stripped_words.as_bytes()).unwrap();
        result_file.write_all(b"\n").unwrap();
        articles_read += 1;

        println!("DEBUG: read {} articles", articles_read);
        
        if (articles_read % 500) == 0 {
            println!("{} articles parsed, {} errorneous articles skipped.",
                articles_read, errorneous_articles);
        }
    }
}


