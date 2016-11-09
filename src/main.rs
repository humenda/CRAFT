extern crate wikipedia2plain;

use std::fs::File;
use std::io::Write;

use wikipedia2plain::*;


fn main() {
    let mut articles_read = 0;
    let mut errorneous_articles = 0;
    let pandoc = pandoc_executor::PandocFilterer::new();
    let mut result_file = File::create("text8").unwrap();
    for article in articles::ArticleParser::new("wikipedia.dump.bz2") {
        if article.starts_with("#REDIRECT") {
            continue;
        }
        articles_read += 1;
        let mut preproc = pandoc_executor::MediawikiPreprocessor::new(&article);
        let result = preproc.preprocess();
        if result.is_err() {
            errorneous_articles += 1;
            continue; // skip article
        }
        let article = pandoc.call_pandoc(&article);
        let stripped_words = text2plain::article2words(article);
        result_file.write_all(stripped_words.as_bytes()).unwrap();
        result_file.write_all(b"\n").unwrap();
        
        if (articles_read % 500) == 0 {
            println!("{} articles parsed, {} errorneous articles skipped.",
                articles_read, errorneous_articles);
        }
    }
}

