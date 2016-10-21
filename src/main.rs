extern crate wikipedia2plain;

use wikipedia2plain::*;

fn main() {
    let mut articles_read = 0;
    let pandoc = pandoc_executor::PandocFilterer::new();
    for article in article_iterator::ArticleIterator::new("wikipedia.dump.bz2") {
        if article.starts_with("#REDIRECT") {
            continue;
        }
        articles_read += 1;
        let mut preproc = pandoc_executor::MediawikiPreprocessor::new(&article);
        // this match must change
        match preproc.preprocess() {
            Ok(x) => {
                let _article = pandoc.call_pandoc(x);
            },
            Err(e) => {
            let text = format!("{:?}\n\\\\\\\\\nFull article:\n{}\n", e, article);
                pandoc_executor::write_error(&text)
            },
        };
        
        if (articles_read % 100) == 0 {
            println!("Article {}", articles_read);
        }
        //println!("article {}: {}\n-----------", articles_read, article);
    }
}

